# status-cli

[![Test Status](https://github.com/amkisko/status-cli.rs/actions/workflows/test.yml/badge.svg)](https://github.com/amkisko/status-cli.rs/actions/workflows/test.yml)

Status page CLI — search the [awesome-status](https://github.com/amkisko/awesome-status) catalog and check live status pages from the terminal.

Companion MCP server: [status_mcp.rb](https://github.com/amkisko/status_mcp.rb).

## Requirements

- Rust 1.70+ (for building from source), or use a pre-built package below.

## Quick start

No API keys. Catalog lookups are offline (bundled data). Live checks use public HTTP.

```bash
cargo install --path status
# or from git
cargo install --git https://github.com/amkisko/status-cli.rs --package status
```

### Install

**Homebrew** (macOS/Linux)

```bash
brew tap amkisko/tap  # once
brew install status-cli
```

**Nix**

```bash
nix build .#default
```

See [packaging/README.md](packaging/README.md) for AUR, Flatpak, FreeBSD, and Gentoo templates.

## Usage

```bash
status search github
status show "GitHub"
status list --limit 20
status check github
status watch --from ~/.config/status/watchlist.toml
status fetch https://www.githubstatus.com
```

### Interactive watch

```bash
status watch github
status watch --from ~/.config/status/watchlist.toml --interval 30
```

Keys: `q` / Esc quit, `r` refresh now, `j`/`k` or arrows select a row.

### Local automation

Watchlist (TOML example at `config/watchlist.example.toml`):

```bash
mkdir -p ~/.config/status
cp config/watchlist.example.toml ~/.config/status/watchlist.toml
status check --from ~/.config/status/watchlist.toml --json \
  --fail-if-degraded --append-jsonl ~/.local/state/status/checks.jsonl
```

- `--from` — file of catalog names or URLs (TOML `targets = [...]`, JSON, or one-per-line)
- `--fail-if-degraded` — exit `3` when any result is degraded or missing a status
- `--append-jsonl` — append one compact JSON object per result (cron-friendly)
- `--interval` / `STATUS_WATCH_INTERVAL` — refresh period for `status watch` (default 30s)

### Output

| Flag / env | Meaning |
|------------|---------|
| `-o plain` / `STATUS_OUTPUT=plain` | Human-readable (default) |
| `--plain` | Script-stable `key=value` lines |
| `-o json` / `--json` | JSON (pretty / compact) |

### Globals

- `--timeout` / `STATUS_TIMEOUT` — HTTP timeout in seconds (default 10)
- `--quiet`, `--verbose`, `--debug`, `--no-color`

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage |
| 3 | Degraded / unknown status (`--fail-if-degraded`) |
| 4 | Network / fetch |
| 5 | I/O |

### Completions and man page

```bash
status completions bash
status man
```

## Development

```bash
make lint
make test
make sync-packaging
```

After updating `status_lib/assets/data.json`, refresh the embedded archive:

```bash
gzip -9 -n -c status_lib/assets/data.json > status_lib/assets/data.json.gz
```

Live provider checks (network; not run in CI):

```bash
make test-live
```

Fetch order: Statuspage `/api/v2/status.json` (+ summary) → incident.io proxy (known hosts) → RSS/Atom → HTML.

## Repository layout

| Path | Role |
|------|------|
| `status/` | CLI binary crate |
| `status_lib/` | Catalog + fetch library |
| `packaging/` | Distro formulas |
| `usr/bin/release/` | Release and packaging sync tooling |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE.md)

## Sponsors

Sponsored by [Kisko Labs](https://kiskolabs.com).
