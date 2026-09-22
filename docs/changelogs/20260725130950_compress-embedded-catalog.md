# Compress embedded catalog

## Participants

- amkisko

## Decisions

- Embed `status_lib/assets/data.json.gz` via `include_bytes!` and decompress with flate2 rust_backend at load time.
- Keep uncompressed `data.json` in-tree as the editable source; regenerate the gzip with `gzip -9 -n`.
- Leave scraper/TUI feature gates for a later pass.

## Effects

- Release binary (arm64 macOS) measured at 5.7 MB before and 5.3 MB after (~0.4 MB / ~7%).
- Catalog unit and CLI search/list/show tests pass with the gzip path.
- Version CLI test now compares against `CARGO_PKG_VERSION`.

## Next

- Feature-gate scraper HTML and optional TUI for further default-binary size cuts.
- Consider a compact binary catalog format if list/search RSS needs to drop further.

## Source

- Resource profile session (status list +1.4 MB RSS vs version; catalog deferred in 20260725123331_release-binary-size.md)
