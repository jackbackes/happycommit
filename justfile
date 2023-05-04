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

# Install dependencies
deps:
    @cargo install cargo-bump

# Bump the patch
bump-patch: deps
    @cargo bump patch --git-tag

# Bump the minor
bump-minor: deps
    @cargo bump minor --git-tag

# Bump the major
bump-major: deps
    @cargo bump major --git-tag
