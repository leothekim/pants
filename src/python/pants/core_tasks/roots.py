# Copyright 2014 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

from pants.task.console_task import ConsoleTask


class ListRoots(ConsoleTask):
  """List the repo's registered source roots."""

  def console_output(self, targets):
    for src_root in self.context.source_roots.all_roots():
      all_langs = ','.join(sorted(src_root.langs))
      yield '{}: {}'.format(src_root.path, all_langs or '*')
