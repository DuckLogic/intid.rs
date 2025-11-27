ALL_STABLE_FEATURES := "idmap/serde,idmap/petgraph_0_8,intid/num-traits,intid/nonmax,intid/bytemuck"

check: && check-format
    cargo +nightly clippy --all-targets --all-features
    cargo +nightly doc --no-deps --all-features

test: check
    cargo +nightly nextest run --all-features
    cargo +stable nextest run --features {{ ALL_STABLE_FEATURES }}

test-exhaustive: test
    cargo +nightly test-all-features

format:
    cargo fmt --all

check-format: && check-spelling
    cargo fmt --check --all

check-spelling: _typos

fix-spelling: (_typos "--write-changes")

_typos *flags:
    # use pinned version to avoid breaking build
    uvx typos@1.36 {{flags}}

