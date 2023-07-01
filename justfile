#!/usr/bin/env just --justfile
set dotenv-load := true

# Build and run.
run *FLAGS:
    cargo run {{FLAGS}}
    just css

watch *FLAGS:
    cargo watch -x run -i output -s 'just css'

css:
    npx tailwindcss -i css/main.css -o output/main.css

# Apply strict formatting.
fmt *FLAGS:
    cargo +nightly fmt  --all {{FLAGS}}

# Run clippy on codebase, tests, examples, while testing all features.
check *FLAGS:
    cargo clippy --tests --examples --all-targets --all-features --workspace {{FLAGS}}

# Run tests.
test *FLAGS:
    cargo nextest run --all-features --workspace {{FLAGS}}

# Generate documentation. Add '-- open' to open the docs in a web page.
doc *FLAGS:
    cargo doc --all-features --document-private-items --workspace 

# Calculate coverage and open page with the results.
coverage *FLAGS:
    cargo llvm-cov {{FLAGS}}

# Benchmark codebase with criterion.
benchmark *FLAGS:
    cargo criterion {{FLAGS}}

# Check for unused dependencies, audit for vulnerabilities,
# and check if newer version of depenedencies is available.
thorough-check:
    cargo +nightly udeps --all-targets
    cargo audit
    cargo upgrades

# Check for unusead features. Opens results in a browser.
unused-features:
    unused-features analyze
    unused-features build-report --input report.json
    rm report.json
    mv report.html /tmp
    xdg-open /tmp/report.html

# Check build timings.
build-timings:
    cargo clean
    cargo build --release --quiet --timings
    xdg-open /target/cargo-timings/cargo-timing.html

# Runs all checks necessary before commit.
# Checks formating, code quality, tests, documentation, spellcheck and more.
pre-commit:
    @just fmt
    @just check -- -D warnings
    @just test
    @just doc
    @just thorough-check
    @just unused-features
    cargo spellcheck fix
    cargo spellcheck reflow

# Similar to `pre-commit` command, but is not interactive and doesn't modify the codebase.
# Suitable for automated CI pipelines.
ci:
    @just fmt --check
    @just check -- -D warnings
    @just test
    @just doc
    @just thorough-check
    cargo spellcheck check

# Initializes the project, installing all tools necessary. Should be run once before begining of development.
init:
    npm install
    echo # installing nightly used by `just fmt` and `cargo udeps`
    rustup install nightly
    echo # things required by `just test`
    cargo install cargo-nextest --no-confirm
    echo # things required by `just watch`
    cargo install cargo-watch --no-confirm
    echo # things required by `just pre-commit`
    cargo install cargo-spellcheck --no-confirm
    echo # things required by `just coverage`
    rustup component add llvm-tools-preview
    cargo install cargo-llvm-cov --no-confirm
    echo # things required by `just benchmark`
    cargo install cargo-criterion --no-confirm
    echo # things required by `just thorough-check`
    cargo install cargo-udeps --no-confirm
    cargo install cargo-audit --no-confirm
    cargo install cargo-upgrades --no-confirm
    cargo install cargo-unused-features --no-confirm
