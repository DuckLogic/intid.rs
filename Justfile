check: && check-format
    cargo +nightly clippy --all-features
    cargo doc --no-deps

test: check
    cargo +nightly test --all-features
    cargo msrv verify --ignore-lockfile

msrv:

test-full: test
    cargo +nightly test-all-features
    cargo +stable test --features serde,petgraph,derive

format:
    cargo fmt --all

check-format: && check-spelling
    cargo fmt --check --all

check-spelling: _typos

fix-spelling: (_typos "--write-changes")

_typos *flags:
    # use pinned version to avoid breaking build
    uvx typos@1.34 {{flags}}

