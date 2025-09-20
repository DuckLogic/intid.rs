ALL_FEATURES := "idmap/nightly,idmap/serde,idmap/petgraph_0_8,intid/num-traits,intid/nonmax,intid/bytemuck"

check: && check-format
    cargo +nightly clippy --all-targets --features {{ ALL_FEATURES }}
    cargo +nightly doc --no-deps --features {{ ALL_FEATURES }}

test: check
    cargo +nightly nextest run --features {{ ALL_FEATURES }}

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
    uvx typos@1.36 {{flags}}

