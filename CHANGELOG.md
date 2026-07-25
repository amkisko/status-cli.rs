# CHANGELOG

## Unreleased

## 0.2.0 (2026-07-25)

- Add `status watch` interactive watchlist dashboard with periodic refresh and keyboard navigation.
- Shrink the release binary with thin LTO, symbol stripping, and a single codegen unit.

## 0.1.0 (2026-07-18)

- Add initial CLI: `search`, `list`, `show`, `check`, `fetch`, `completions`, `man`, `version`.
- Add bundled offline catalog from awesome-status (via status_mcp data).
- Add live fetch pipeline: Statuspage `/api/v2` JSON, incident.io proxy, RSS/Atom, HTML heuristics.
- Add human, plain, and JSON output modes with script-friendly exit codes.
- Add packaging templates: Homebrew, Nix, AUR, Flatpak, FreeBSD, Gentoo.
- Prefer Statuspage public JSON for overall status (accurate degraded/outage indicators).
- Map incident.io overall status from affected components, not incident lifecycle words.
- Suppress HTML crawler/JS errors when API or feed already returned usable status.
- Add `check --from <watchlist>` for local name/URL lists (TOML, JSON, or line file).
- Add `--fail-if-degraded` on `check`/`fetch` (exit code 3).
- Add `--append-jsonl <file>` for cron-friendly local logs.
