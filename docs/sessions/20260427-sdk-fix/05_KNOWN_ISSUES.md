# Known Issues

- `ShellExecuteExW` cannot accept a custom environment block. The elevated setup launch now uses
  a temporary process-wide env scrub as a best-effort mitigation and restores values immediately
  after the call.
- The env scrub is process-wide and temporary, so any concurrent code in the same process would
  observe the scrubbed values during the launch window. That is still preferable to inheriting
  the unsafe injection variables into the elevated helper.
