check: && check-format
    cargo clippy --all-features

format:
    cargo fmt --all

check-format:
    cargo fmt --check --all


