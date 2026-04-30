# JavaScript REPL (`js_repl`)

`js_repl` runs JavaScript in a persistent Node-backed kernel with top-level `await`.

## Summary

- Use dynamic `await import(...)` for module loading.
- Top-level bindings persist across calls until `js_repl_reset`.
- Use `codex.tool(...)` to reach other Codex tools from inside the kernel.
- Use `codex.emitImage(...)` only when you need to pass image output back to the model.

## Notes

- The kernel uses a JSON-line transport over stdio.
- Avoid writing directly to `process.stdout`, `process.stderr`, or `process.stdin`.
- Local file modules reload between execs.

