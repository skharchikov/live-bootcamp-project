# Format and lint all services
fmt-all:
    cargo fmt --all --manifest-path auth-service/Cargo.toml
    cargo fmt --all --manifest-path app-service/Cargo.toml
    cargo clippy --manifest-path auth-service/Cargo.toml -- -D warnings
    cargo clippy --manifest-path app-service/Cargo.toml -- -D warnings

# Format and lint a specific service
fmt SERVICE:
    cargo fmt --manifest-path {{SERVICE}}-service/Cargo.toml
    cargo clippy --manifest-path {{SERVICE}}-service/Cargo.toml -- -D warnings

# Run all tests for all services
test-all:
    cargo test --manifest-path auth-service/Cargo.toml
    cargo test --manifest-path app-service/Cargo.toml
