# Copyright 2014 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

target(
  name='all',
  dependencies=[
    ':sitegen',
  ],
)

python_library(
  name='sitegen',
  sources=['sitegen.py'],
  dependencies=[
    'pants-plugins/3rdparty/python:beautifulsoup4',
    'src/python/pants/base:exceptions',
    'src/python/pants/task',
    'src/python/pants/backend/docgen/tasks'
  ]
)
