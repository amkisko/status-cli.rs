## Participants

amkisko

## Decisions

Prefer Atlassian Statuspage public JSON (/api/v2/status.json and summary.json) before RSS/HTML for overall status.
Keep incident.io for known hosts as a secondary API; map overall status from affected_components.
Suppress HTML crawler and JavaScript-shell errors when an API or feed already returned usable status.
Keep live provider tests behind cargo test --ignored so CI stays offline-stable.

## Effects

Statuspage provider landed with fixture unit tests and ignored live tests.
Live probe of 16 popular services: 15 returned clean status; 14 via Statuspage API; Cloudflare/Twilio/Zoom report degraded correctly; GitHub check dropped from ~73s to ~1.5s.
Notion remains unresolved (JS-only page, no Statuspage or incident.io JSON).

## Next

Optional: Google Cloud / AWS custom parsers if catalog demand justifies it.
Optional: fail check/fetch with a clearer exit when no status and no hard error (Notion case).

## Source

Provider accuracy probe against Statuspage /api/v2/status.json and status_mcp FetchStatusTool pipeline.
