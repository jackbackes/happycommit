# Define a default 'all' task that runs 'fix' task
default:
    @just --list

# Format Rust code using rustfmt
format:
    @cargo fmt

# Run Clippy to check for common mistakes
clippy:
    @cargo clippy

# Install the binary
install:
    @cargo install --path .

# Run both 'format' and 'clippy' tasks
fix: format clippy