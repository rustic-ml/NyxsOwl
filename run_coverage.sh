#!/bin/bash

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null
then
    echo "cargo-tarpaulin could not be found, installing..."
    cargo install cargo-tarpaulin
fi

# Run workspace-level coverage
echo "Running workspace-level coverage..."
cargo tarpaulin --verbose --workspace --timeout 120 --out Html --out Xml

# Run specific coverage for forecast_trade crate
echo "Running forecast_trade crate coverage..."
cd forecast_trade && cargo tarpaulin --verbose --timeout 120 --out Html --out Xml --output-dir coverage

echo "Code coverage reports generated:"
echo "- Workspace: tarpaulin-report.html"
echo "- forecast_trade: forecast_trade/coverage/tarpaulin-report.html"

# Optional: Upload to codecov if CI
if [ "$CI" = "true" ]; then
    echo "Uploading coverage reports to codecov..."
    bash <(curl -s https://codecov.io/bash) -f tarpaulin-report.xml
    bash <(curl -s https://codecov.io/bash) -f forecast_trade/coverage/tarpaulin-report.xml -F forecast_trade
fi 