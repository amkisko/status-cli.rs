## Participants

The repository maintainer and Cursor agent.

## Decisions

HTML scraping is an optional status_lib feature. The status CLI enables HTML scraping and the interactive watch dashboard by default, while a no-default-features build omits scraper and ratatui.

RSS and Atom parsing with feed-rs remains enabled in every build.

## Effects

The default release binary measured 5.3M. The no-default-features release binary measured 4.4M.

Validated with cargo fmt --all -- --check, cargo test --workspace, cargo test -p status --no-default-features, and cargo test -p status_lib --no-default-features.

## Next

No follow-up is planned.

## Source

CHANGELOG.md records the user-facing release note. This changelog records the feature-gate decision and release-size measurements.
