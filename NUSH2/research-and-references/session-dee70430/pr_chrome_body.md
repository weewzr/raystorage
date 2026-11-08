## Summary

The browser self test failed three site builds on GitHub's runner before any check ran: headless Chrome did not answer the DevTools list within about 20 s, so the build stopped and the deploy was skipped. This makes the Chrome start sturdier. The checks themselves are unchanged.

- Chrome gets `--remote-debugging-port=0` and picks a free port itself. The test reads it from `DevToolsActivePort` in the profile folder, or from the `DevTools listening on` log line, so no port is chosen in advance.
- Each start may take 45 s. A start that shows no page target is stopped, its log tail is printed, and Chrome starts again with a fresh profile, up to three starts.
- If every start fails, the test prints `FAIL` with the reason instead of a traceback.

## Checks that ran

- `selftest.py` on a built site of the tour revision passes at 1440x900 and 375x812 (mobile), each with and without reduced motion: 33 pass, 1 skip, 0 fail, 0 browser errors.
- The same patched script passed all four runs against the live site's deployed files.
- Failure path: a command that exits at once gives two printed failed starts, then a clean `TimeoutError`.
- Retry path: after a deliberately failed start in the same folder, the next start finds its page target in 0.35 s.

`scripts/wasm_smoke.py` still picks its port in advance, but no workflow runs it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
