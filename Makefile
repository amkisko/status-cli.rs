.PHONY: lint build install fmt clippy check-loc test test-live sync-packaging check-packaging release bump-homebrew

CARGO ?= cargo
HOMEBREW_TAP ?= $(abspath ../homebrew-tap)
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TAG ?= v$(VERSION)

lint: fmt clippy check-loc

fmt:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

check-loc:
	$(CARGO) run -p release --bin check-loc

build:
	$(CARGO) build --workspace

test:
	$(CARGO) test --workspace

test-live:
	$(CARGO) test -p status_lib -- --ignored --nocapture

install:
	$(CARGO) install --path status --locked

sync-packaging:
	$(CARGO) run -p release --bin sync-packaging

check-packaging:
	$(CARGO) run -p release --bin sync-packaging -- --check

release:
	$(CARGO) run -p release

# Requires the GitHub tag to exist. Updates sibling homebrew-tap and packaging formula.
bump-homebrew:
	@test -n "$(VERSION)" || (echo "could not read version from Cargo.toml" >&2; exit 2)
	$(HOMEBREW_TAP)/scripts/bump-formula.sh \
		--formula status-cli \
		--tag "$(TAG)" \
		--repository amkisko/status-cli.rs \
		--mirror "$(CURDIR)/packaging/homebrew/status-cli.rb" \
		--commit
