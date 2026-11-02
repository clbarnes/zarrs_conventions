_:
    just --list

test-all:
    cargo test --all-features --all-targets --workspace

lint-all:
    cargo clippy --all-features --all-targets --workspace -- -D warnings

fmt-all:
    cargo fmt --all
