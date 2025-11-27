ALL_STABLE_FEATURES := "idmap/serde,idmap/petgraph_0_8,intid/num-traits,intid/nonmax,intid/bytemuck"

check: && check-format
    cargo +nightly clippy --all-targets --all-features
    cargo +nightly doc --no-deps --all-features
    # Checking MSRV
    cargo +1.65 check --all-targets --features {{ ALL_STABLE_FEATURES }}

test: check
    cargo +nightly nextest run --all-features
    cargo +stable nextest run --features {{ ALL_STABLE_FEATURES }}
    # Testing MSRV
    cargo +1.65 nextest run --features {{ ALL_STABLE_FEATURES }}
    # Test that things work with the expander
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

