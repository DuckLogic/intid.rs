check: && check-format
    cargo clippy --all-features

format:
    cargo fmt --all

check-format: && check-spelling
    cargo fmt --check --all

check-spelling: _typos

fix-spelling: (_typos "--write-changes")

_typos *flags:
    # use pinned version to avoid breaking build
    uvx typos@1.34 {{flags}}

