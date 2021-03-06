// Copyright 2017 Pants project contributors (see CONTRIBUTORS.md).
// Licensed under the Apache License, Version 2.0 (see LICENSE).

#![deny(warnings)]
// Enable all clippy lints except for many of the pedantic ones. It's a shame this needs to be copied and pasted across crates, but there doesn't appear to be a way to include inner attributes from a common source.
#![deny(
  clippy::all,
  clippy::default_trait_access,
  clippy::expl_impl_clone_on_copy,
  clippy::if_not_else,
  clippy::needless_continue,
  clippy::single_match_else,
  clippy::unseparated_literal_suffix,
  clippy::used_underscore_binding
)]
// It is often more clear to show that nothing is being moved.
#![allow(clippy::match_ref_pats)]
// Subjective style.
#![allow(
  clippy::len_without_is_empty,
  clippy::redundant_field_names,
  clippy::too_many_arguments
)]
// Default isn't as big a deal as people seem to think it is.
#![allow(clippy::new_without_default, clippy::new_ret_no_self)]
// Arc<Mutex> can be more clear than needing to grok Orderings:
#![allow(clippy::mutex_atomic)]

use bytes::Bytes;
use futures::Future;
use hashing::Fingerprint;
use lmdb::{
  self, Database, DatabaseFlags, Environment, EnvironmentCopyFlags, EnvironmentFlags,
  RwTransaction, Transaction, WriteFlags,
};
use log::trace;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time;
use tempfile::TempDir;

// Each LMDB directory can have at most one concurrent writer.
// We use this type to shard storage into 16 LMDB directories, based on the first 4 bits of the
// fingerprint being stored, so that we can write to them in parallel.
#[derive(Clone)]
pub struct ShardedLmdb {
  // First Database is content, second is leases.
  lmdbs: HashMap<u8, (Arc<Environment>, Database, Database)>,
  root_path: PathBuf,
  max_size_bytes: usize,
  executor: task_executor::Executor,
}

impl ShardedLmdb {
  // max_size is the maximum size the databases together will be allowed to grow to in bytes.
  // When calling this function, we will attempt to allocate that much virtual (not resident) memory
  // for the mmap; in theory it should be possible not to bound this, but in practice we see travis
  // occasionally fail tests because it's unable to allocate virtual memory if we set this too high,
  // and we have too many tests running concurrently or close together.
  pub fn new(
    root_path: PathBuf,
    max_size_bytes: usize,
    executor: task_executor::Executor,
  ) -> Result<ShardedLmdb, String> {
    trace!("Initializing ShardedLmdb at root {:?}", root_path);
    let mut lmdbs = HashMap::new();

    for (env, dir, fingerprint_prefix) in ShardedLmdb::envs(&root_path, max_size_bytes)? {
      trace!("Making ShardedLmdb content database for {:?}", dir);
      let content_database = env
        .create_db(Some("content"), DatabaseFlags::empty())
        .map_err(|e| {
          format!(
            "Error creating/opening content database at {:?}: {}",
            dir, e
          )
        })?;

      trace!("Making ShardedLmdb lease database for {:?}", dir);
      let lease_database = env
        .create_db(Some("leases"), DatabaseFlags::empty())
        .map_err(|e| {
          format!(
            "Error creating/opening content database at {:?}: {}",
            dir, e
          )
        })?;

      lmdbs.insert(
        fingerprint_prefix,
        (Arc::new(env), content_database, lease_database),
      );
    }

    Ok(ShardedLmdb {
      lmdbs,
      root_path,
      max_size_bytes,
      executor,
    })
  }

  fn envs(
    root_path: &Path,
    max_size_bytes: usize,
  ) -> Result<Vec<(Environment, PathBuf, u8)>, String> {
    let mut envs = Vec::with_capacity(0x10);
    for b in 0x00..0x10 {
      let fingerprint_prefix = b << 4;
      let mut dirname = String::new();
      fmt::Write::write_fmt(&mut dirname, format_args!("{:x}", fingerprint_prefix)).unwrap();
      let dirname = dirname[0..1].to_owned();
      let dir = root_path.join(dirname);
      fs::safe_create_dir_all(&dir)
        .map_err(|err| format!("Error making directory for store at {:?}: {:?}", dir, err))?;
      envs.push((
        ShardedLmdb::make_env(&dir, max_size_bytes)?,
        dir,
        fingerprint_prefix,
      ));
    }
    Ok(envs)
  }

  fn make_env(dir: &Path, max_size_bytes: usize) -> Result<Environment, String> {
    Environment::new()
      // NO_SYNC
      // =======
      //
      // Don't force fsync on every lmdb write transaction
      //
      // This significantly improves performance on slow or contended disks.
      //
      // On filesystems which preserve order of writes, on system crash this may lead to some
      // transactions being rolled back. This is fine because this is just a write-once
      // content-addressed cache. There is no risk of corruption, just compromised durability.
      //
      // On filesystems which don't preserve the order of writes, this may lead to lmdb
      // corruption on system crash (but in no other circumstances, such as process crash).
      //
      // ------------------------------------------------------------------------------------
      //
      // NO_TLS
      // ======
      //
      // Without this flag, each time a read transaction is started, it eats into our
      // transaction limit (default: 126) until that thread dies.
      //
      // This flag makes transactions be removed from that limit when they are dropped, rather
      // than when their thread dies. This is important, because we perform reads from a
      // thread pool, so our threads never die. Without this flag, all read requests will fail
      // after the first 126.
      //
      // The only down-side is that you need to make sure that any individual OS thread must
      // not try to perform multiple write transactions concurrently. Fortunately, this
      // property holds for us.
      .set_flags(EnvironmentFlags::NO_SYNC | EnvironmentFlags::NO_TLS)
      // 2 DBs; one for file contents, one for leases.
      .set_max_dbs(2)
      .set_map_size(max_size_bytes / page_size::get())
      .open(dir)
      .map_err(|e| format!("Error making env for store at {:?}: {}", dir, e))
  }

  // First Database is content, second is leases.
  pub fn get(&self, fingerprint: &Fingerprint) -> (Arc<Environment>, Database, Database) {
    self.lmdbs[&(fingerprint.0[0] & 0xF0)].clone()
  }

  pub fn all_lmdbs(&self) -> Vec<(Arc<Environment>, Database, Database)> {
    self.lmdbs.values().cloned().collect()
  }

  pub fn store_bytes(
    &self,
    key: Fingerprint,
    bytes: Bytes,
    initial_lease: bool,
  ) -> impl Future<Item = (), Error = String> {
    let store = self.clone();
    self
      .executor
      .spawn_on_io_pool(futures::future::lazy(move || {
        let (env, db, lease_database) = store.get(&key);
        let put_res = env.begin_rw_txn().and_then(|mut txn| {
          txn.put(db, &key, &bytes, WriteFlags::NO_OVERWRITE)?;
          if initial_lease {
            store.lease(
              lease_database,
              &key,
              Self::default_lease_until_secs_since_epoch(),
              &mut txn,
            )?;
          }
          txn.commit()
        });

        match put_res {
          Ok(()) => Ok(()),
          Err(lmdb::Error::KeyExist) => Ok(()),
          Err(err) => Err(format!("Error storing key {:?}: {}", key.to_hex(), err)),
        }
      }))
  }

  fn lease(
    &self,
    database: Database,
    fingerprint: &Fingerprint,
    until_secs_since_epoch: u64,
    txn: &mut RwTransaction<'_>,
  ) -> Result<(), lmdb::Error> {
    txn.put(
      database,
      &fingerprint.as_ref(),
      &until_secs_since_epoch.to_le_bytes(),
      WriteFlags::empty(),
    )
  }

  fn default_lease_until_secs_since_epoch() -> u64 {
    let now_since_epoch = time::SystemTime::now()
      .duration_since(time::UNIX_EPOCH)
      .expect("Surely you're not before the unix epoch?");
    (now_since_epoch + time::Duration::from_secs(2 * 60 * 60)).as_secs()
  }

  pub fn load_bytes_with<
    T: Send + 'static,
    F: Fn(Bytes) -> Result<T, String> + Send + Sync + 'static,
  >(
    &self,
    fingerprint: Fingerprint,
    f: F,
  ) -> impl Future<Item = Option<T>, Error = String> {
    let store = self.clone();
    self
      .executor
      .spawn_on_io_pool(futures::future::lazy(move || {
        let (env, db, _) = store.get(&fingerprint);
        let ro_txn = env
          .begin_ro_txn()
          .map_err(|err| format!("Failed to begin read transaction: {}", err));
        ro_txn.and_then(|txn| match txn.get(db, &fingerprint) {
          Ok(bytes) => f(Bytes::from(bytes)).map(Some),
          Err(lmdb::Error::NotFound) => Ok(None),
          Err(err) => Err(format!(
            "Error loading fingerprint {:?}: {}",
            fingerprint.to_hex(),
            err,
          )),
        })
      }))
  }

  #[allow(clippy::identity_conversion)] // False positive: https://github.com/rust-lang/rust-clippy/issues/3913
  pub fn compact(&self) -> Result<(), String> {
    for (env, old_dir, _) in ShardedLmdb::envs(&self.root_path, self.max_size_bytes)? {
      let new_dir = TempDir::new_in(old_dir.parent().unwrap()).expect("TODO");
      env
        .copy(new_dir.path(), EnvironmentCopyFlags::COMPACT)
        .map_err(|e| {
          format!(
            "Error copying store from {:?} to {:?}: {}",
            old_dir,
            new_dir.path(),
            e
          )
        })?;
      std::fs::remove_dir_all(&old_dir)
        .map_err(|e| format!("Error removing old store at {:?}: {}", old_dir, e))?;
      std::fs::rename(&new_dir.path(), &old_dir).map_err(|e| {
        format!(
          "Error replacing {:?} with {:?}: {}",
          old_dir,
          new_dir.path(),
          e
        )
      })?;

      // Prevent the tempdir from being deleted on drop.
      std::mem::drop(new_dir);
    }
    Ok(())
  }
}
