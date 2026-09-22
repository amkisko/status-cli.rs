## Participants

amkisko

## Decisions

Ship status-cli.rs as a Rust workspace mirroring scout-cli structure, with binary status, offline awesome-status catalog, and live fetch parity with status_mcp.rb.

## Effects

Initial 0.1.0 scaffold: search, list, show, check, fetch, packaging templates, CI, and community docs.

## Next

Tag first release after CI is green; fill Homebrew sha256 and AUR checksums when the tag exists.

## Source

Plan: status-cli scaffold from status_mcp.rb features and scout-cli.rs layout.
