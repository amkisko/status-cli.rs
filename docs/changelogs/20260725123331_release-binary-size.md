# Release binary size

## Participants

- amkisko

## Decisions

- Add workspace `[profile.release]` with thin LTO, codegen-units = 1, strip, and panic = abort.
- Leave catalog compression and TUI feature gates for a later pass.

## Effects

- Release builds strip symbols and apply thin LTO by default.
- Measured arm64 macOS release size: 9.5 MB before, 5.7 MB after (-40%).

## Next

- Optional: compress or externalize embedded catalog; feature-gate scraper HTML path.

## Source

- Binary size analysis session (timely, scout, status, pray)
