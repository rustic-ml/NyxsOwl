# Testing and Code Coverage

This document describes how to run tests and generate code coverage reports for the `forecast_trade` crate.

## Running Tests

To run the tests for this crate:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests in a specific file
cargo test --test test_file_name
```

## Code Coverage

This crate uses [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) for code coverage analysis.

### Installing Tarpaulin

```bash
cargo install cargo-tarpaulin
```

### Generating Coverage Reports

```bash
# Generate HTML report
cargo tarpaulin --out Html

# Generate XML report (for CI)
cargo tarpaulin --out Xml

# Generate both formats
cargo tarpaulin --out Html --out Xml
```

### Running Coverage with Custom Options

```bash
# Ignore specific files
cargo tarpaulin --exclude-files "src/bin/*"

# Set minimum coverage threshold
cargo tarpaulin --min-coverage 90

# Run with verbose output
cargo tarpaulin --verbose
```

## Continuous Integration

The code coverage is automatically checked in CI using GitHub Actions. The workflow is defined in `.github/workflows/forecast_trade_coverage.yml`.

The coverage results are also uploaded to [Codecov](https://codecov.io) for visualization and tracking over time.

## Coverage Requirements

We aim to maintain at least 90% code coverage for this crate. When adding new features, please ensure:

1. All public functions have associated tests
2. All error paths are tested
3. Edge cases are covered by tests
4. Integration tests demonstrate end-to-end functionality

## Test Organization

The tests are organized as follows:

- Unit tests: Located alongside the code they test (in the same file)
- Integration tests: Located in the `tests/` directory
- Each module has its own test file in the `tests/` directory

## Running All Tests with Coverage

You can use the provided script to run all tests with coverage:

```bash
./run_coverage.sh
```

This will generate both a workspace-level coverage report and a crate-specific report. 