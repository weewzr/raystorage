## Summary

Adds a guided process tour to the Interactive PFD page. A button above the drawing starts a 13 step walk through the plant: fission heat in the HTGR core, the primary and secondary helium loops, the reformer train, MEA capture, the PSA and the H2 product, the reject recycle, CO2 compression and ship storage, and the power block. Each step lights its units, moves dashes along its streams in the flow direction and fills a caption card with numbers read from the live engine reply.

- Every caption number comes from the live reply. No precomputed rows remain.
- On an infeasible plant the captions separate what the plant needs from what it gets.
- Controls sit outside the drawing: Start, Play or Pause, Previous, Next and Exit, with arrow keys, Space and Escape. Each step is announced through a live region.
- Under reduced motion the tour is still and does not advance by itself.
- Scroll rule: an automatic step never scrolls the page or the side column. A step the reader asks for keeps the control used in view with the smallest scroll, and the card head too when both fit.
- No third-party code and no new dependency. Nothing under `engine/` or `results/` changes.

## Checks that ran

On the final revision 83e4f6d, from a clean build of the wasm module and the site:

- `selftest.py` passes at 1440x900 and 375x812 (mobile), each with and without reduced motion: 33 pass, 1 skip, 0 fail, 0 browser errors, 0 requests to another host. 1024x768 also passes.
- The new self test check `tour_auto_stays` fails if an automatic step changes the page scroll or the side column scroll. A planted copy of the previous tour fails it, so the check catches the fault it targets.
- Scroll and focus scripts at 13 window sizes, with and without reduced motion: 0 px page movement on automatic steps in 52 of 52 runs, and no hidden card head where head and control fit.
- Caption numbers: 1155 numbers across 12 input sets match the engine reply (round 2 audit, unchanged since).
- Site and repository text scans pass, and the pre-push scan of the full history reports 0 hits.

## Not done

- The independent recheck of this last fix round was stopped before it reported, so this revision is checked by the fix round's own runs and by CI only.
- At 1024x600 the self test check `tour_keyboard` fails. It failed the same way before this round. CI runs 1440x900, where it passes.
- When the reader is scrolled to the unit panel below the card, Chrome's own scroll anchoring moves the page on automatic steps to hold that panel in place. The tour itself calls no scroll there.
- At 1440x900, Previous back to step 1 keeps the button in view but not the whole card header, since the two do not fit together.
- Drawing details still to match the engine: the drawing's labels show need values on an infeasible plant, the reformer helium is drawn co-current where the engine has counterflow, and a secondary circulator, the reboiler heat lines and the burner are not drawn.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
