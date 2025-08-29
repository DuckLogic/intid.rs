check: && check-format
    cargo +nightly clippy --all-targets --all-features
    cargo doc --no-deps --all-features

test: check
    cargo +nightly test --all-features

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
    uvx typos@1.35 {{flags}}

