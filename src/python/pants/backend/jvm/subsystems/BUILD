# Copyright 2015 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

python_library(
  name = 'dependency_context',
  sources = ['dependency_context.py'],
  dependencies = [
    ':java',
    ':scala_platform',
    'src/python/pants/backend/codegen/thrift/java',
    'src/python/pants/base:fingerprint_strategy',
    'src/python/pants/build_graph',
    'src/python/pants/java/jar',
  ],
)

python_library(
  name = 'jvm_tool_mixin',
  sources = ['jvm_tool_mixin.py'],
  dependencies = [
    'src/python/pants/base:exceptions',
    'src/python/pants/java/distribution',
    'src/python/pants/option',
  ],
)

python_library(
  name = 'jar_tool',
  sources = ['jar_tool.py'],
  dependencies = [
    ':jvm_tool_mixin',
    'src/python/pants/java/jar',
    'src/python/pants/base:workunit',
    'src/python/pants/option',
    'src/python/pants/subsystem',
  ],
)

python_library(
  name = 'java',
  sources = ['java.py'],
  dependencies = [
    ':zinc_language_mixin',
    'src/python/pants/backend/jvm/subsystems:jvm_tool_mixin',
    'src/python/pants/build_graph',
    'src/python/pants/option',
    'src/python/pants/subsystem',
  ],
)

python_library(
  name = 'junit',
  sources = ['junit.py'],
  dependencies = [
    ':jvm_tool_mixin',
    'src/python/pants/java/jar',
    'src/python/pants/backend/jvm/subsystems:shader',
    'src/python/pants/util:memo',
    'src/python/pants/build_graph',
    'src/python/pants/subsystem',
  ],
)

python_library(
  name = 'jvm',
  sources = ['jvm.py'],
  dependencies = [
    'src/python/pants/option',
    'src/python/pants/subsystem',
    'src/python/pants/util:strutil',
  ],
)

python_library(
  name = 'jvm_platform',
  sources = ['jvm_platform.py'],
  dependencies = [
    'src/python/pants/base:exceptions',
    'src/python/pants/base:revision',
    'src/python/pants/option',
    'src/python/pants/subsystem',
    'src/python/pants/util:memo',
    'src/python/pants/java/distribution',
  ],
)

python_library(
  name = 'jar_dependency_management',
  sources = ['jar_dependency_management.py'],
  dependencies = [
    'src/python/pants/backend/jvm/targets:jvm',
    'src/python/pants/base:revision',
    'src/python/pants/build_graph',
    'src/python/pants/option',
    'src/python/pants/subsystem',
    'src/python/pants/task',
  ],
)

python_library(
  name = 'scala_platform',
  sources = ['scala_platform.py'],
  dependencies = [
    ':jvm_tool_mixin',
    ':zinc_language_mixin',
    'src/python/pants/option',
    'src/python/pants/subsystem',
    'src/python/pants/java/jar',
    'src/python/pants/backend/jvm/targets:jvm',
    'src/python/pants/util:memo',
  ],
)

python_library(
  name = 'shader',
  sources = ['shader.py'],
  dependencies = [
    'src/python/pants/java/jar',
    'src/python/pants/backend/jvm/tasks:classpath_util',
    'src/python/pants/backend/jvm/tasks:jvm_tool_task_mixin',
    'src/python/pants/java/distribution',
    'src/python/pants/java:executor',
    'src/python/pants/subsystem',
    'src/python/pants/util:contextutil',
    'src/python/pants/util:memo',
    'src/python/pants/util:objects',
  ]
)

python_library(
  name = 'zinc',
  sources = ['zinc.py'],
  dependencies = [
    ':dependency_context',
    ':jvm_tool_mixin',
    ':shader',
    'src/python/pants/subsystem',
    'src/python/pants/util:dirutil',
  ]
)

python_library(
  name = 'zinc_language_mixin',
  sources = ['zinc_language_mixin.py'],
  dependencies = [
    'src/python/pants/build_graph',
    'src/python/pants/subsystem',
  ]
)

python_library(
  name='resolve_subsystem',
  sources=['resolve_subsystem.py'],
  dependencies=[
    'src/python/pants/subsystem',
  ],
)

python_library(
  name='rsc',
  sources=['rsc.py'],
  dependencies=[
    ':jvm_tool_mixin',
    ':shader',
    ':zinc',
    'src/python/pants/subsystem',
    'src/python/pants/util:dirutil',
  ],
)
