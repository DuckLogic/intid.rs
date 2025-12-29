ALL_STABLE_FEATURES := "idmap/serde,idmap/petgraph_0_8,intid/num-traits,intid/nonmax,intid/bytemuck"

check: && check-format
    cargo +nightly clippy --all-targets --all-features
    cargo +nightly doc --no-deps --all-features
    @# TODO: Go back to checking MSRV (would require regenerating Cargo.lock

test: check
    cargo +nightly nextest run --all-features
    cargo +stable nextest run --features {{ ALL_STABLE_FEATURES }}
    @# TODO: Go back to testing  MSRV (see above)
    RUSTFLAGS="--cfg intid_derive_use_expander" cargo +nightly nextest run --all-features


test-exhaustive: test
    cargo +nightly all-features nextest run --no-tests=warn

format:
    cargo fmt --all

check-format: && check-spelling
    cargo fmt --check --all

check-spelling: _typos

fix-spelling: (_typos "--write-changes")

_typos *flags:
    # use pinned version to avoid breaking build
    uvx typos@1.36 {{flags}}

