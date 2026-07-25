## Participants

amkisko

## Decisions

Add thin local automation only: watchlist file, --fail-if-degraded (exit 3), and --append-jsonl.
Do not add a daemon, decentralised reporting, or incident store in this pass.

## Effects

check accepts --from watchlist (TOML/JSON/lines), optional --fail-if-degraded and --append-jsonl.
fetch accepts the same fail/jsonl flags.
Example watchlist at config/watchlist.example.toml.
Operational classification lives in status_lib for unit tests.

## Next

Optional ignored live test that asserts exit 3 against a currently degraded Statuspage host.
Reassess before any statusd or alert routing layer.

## Source

Product decision: watchlist + fail-if-degraded + jsonl append as the first user-side check config.
