# Copyright 2019 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

from pants.base.payload import Payload
from pants.base.payload_field import PrimitiveField, PrimitivesSetField
from pants.build_graph.target import Target


class PackagedNativeLibrary(Target):
  """A container for headers and libraries from external sources.

  This target type is intended to be generated by a codegen task to wrap various sources of C/C++
  packages in a homogenous container. It can also be used to wrap native libraries which are checked
  into the repository -- the `sources` argument does not allow files outside of the buildroot.
  """

  @classmethod
  def alias(cls):
    return 'packaged_native_library'

  def __init__(self, address, payload=None, sources=None, include_relpath=None, lib_relpath=None,
               native_lib_names=None, **kwargs):
    """
    :param sources: Files owned by this target.
    :param str include_relpath: The path where C/C++ headers are located, relative to this target's
                                directory. Libraries depending on this target will be able to
                                #include files relative to this directory.
    :param str lib_relpath: The path where native libraries are located, relative to this target's
                            directory.
    :param list native_lib_names: Strings containing the libraries to add to the linker command
                                  line. These libraries become `-l<name>` arguments, so they must
                                  exist and be named `lib<name>.so` (or `lib<name>.dylib` depending
                                  on the platform) or the linker will exit with an error.
    """
    if not payload:
      payload = Payload()
    payload.add_fields({
      'sources': self.create_sources_field(sources, address.spec_path, key_arg='sources'),
      'include_relpath': PrimitiveField(include_relpath),
      'lib_relpath': PrimitiveField(lib_relpath),
      'native_lib_names': PrimitivesSetField(native_lib_names),
    })
    super().__init__(address=address, payload=payload, **kwargs)

  @property
  def include_relpath(self):
    return self.payload.include_relpath

  @property
  def lib_relpath(self):
    return self.payload.lib_relpath

  @property
  def native_lib_names(self):
    return self.payload.native_lib_names
