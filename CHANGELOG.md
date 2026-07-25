# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `status watch` interactive watchlist dashboard (ratatui): periodic refresh, keyboard navigation

### Changed

- Shrink the release binary with thin LTO, symbol stripping, and a single codegen unit

## [0.1.0] - 2026-07-18

### Added

- Initial CLI: `search`, `list`, `show`, `check`, `fetch`, `completions`, `man`, `version`
- Bundled offline catalog from awesome-status (via status_mcp data)
- Live fetch pipeline: Statuspage `/api/v2` JSON, incident.io proxy, RSS/Atom, HTML heuristics
- Human, plain, and JSON output modes with script-friendly exit codes
- Packaging templates: Homebrew, Nix, AUR, Flatpak, FreeBSD, Gentoo

### Changed

- Prefer Statuspage public JSON for overall status (accurate degraded/outage indicators)
- Map incident.io overall status from affected components, not incident lifecycle words
- Suppress HTML crawler/JS errors when API or feed already returned usable status

### Added

- `check --from <watchlist>` for local name/URL lists (TOML, JSON, or line file)
- `--fail-if-degraded` on `check`/`fetch` (exit code 3)
- `--append-jsonl <file>` for cron-friendly local logs
