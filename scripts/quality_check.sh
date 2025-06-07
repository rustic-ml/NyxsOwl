#!/bin/bash
# Quality Assurance Script for NyxsOwl Trading Library

set -e  # Exit on any error

echo "🎯 Starting NyxsOwl Quality Assurance Check..."
echo "==============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
    else
        echo -e "${RED}✗ $2${NC}"
    fi
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Initialize counters
TOTAL_CHECKS=0
PASSED_CHECKS=0

# 1. Compilation Check
echo "1. 🔨 Checking Compilation..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if cargo check --features forecasting --quiet; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    print_status 0 "Compilation successful"
else
    print_status 1 "Compilation failed"
fi

# 2. Test Suite
echo -e "\n2. 🧪 Running Test Suite..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if cargo test --features forecasting --quiet; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    print_status 0 "Test suite passed"
    
    # Get test statistics
    TEST_OUTPUT=$(cargo test --features forecasting 2>&1)
    PASSED_TESTS=$(echo "$TEST_OUTPUT" | grep -o '[0-9]* passed' | head -1 | awk '{print $1}')
    FAILED_TESTS=$(echo "$TEST_OUTPUT" | grep -o '[0-9]* failed' | head -1 | awk '{print $1}')
    
    if [ -n "$PASSED_TESTS" ]; then
        echo "   📊 Tests passed: $PASSED_TESTS"
    fi
    if [ -n "$FAILED_TESTS" ] && [ "$FAILED_TESTS" -gt 0 ]; then
        print_warning "Tests failed: $FAILED_TESTS"
    fi
else
    print_status 1 "Test suite failed"
fi

# 3. Documentation Generation
echo -e "\n3. 📚 Checking Documentation..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if cargo doc --features forecasting --no-deps --quiet; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    print_status 0 "Documentation generated successfully"
else
    print_status 1 "Documentation generation failed"
fi

# 4. Lint Check (Clippy)
echo -e "\n4. 🔍 Running Clippy Lints..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if cargo clippy --features forecasting --quiet -- -D warnings; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    print_status 0 "Clippy lints passed"
else
    print_status 1 "Clippy lints failed"
    print_warning "Run 'cargo clippy --features forecasting' for details"
fi

# 5. Format Check
echo -e "\n5. 📝 Checking Code Format..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if cargo fmt --check; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    print_status 0 "Code formatting is correct"
else
    print_status 1 "Code formatting issues found"
    print_warning "Run 'cargo fmt' to fix formatting"
fi

# 6. Examples Check
echo -e "\n6. 🚀 Testing Examples..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
EXAMPLES_PASSED=true

# Test quick_start example
if cargo run --example quick_start --quiet; then
    echo -e "   ${GREEN}✓ quick_start example works${NC}"
else
    echo -e "   ${RED}✗ quick_start example failed${NC}"
    EXAMPLES_PASSED=false
fi

# Test basic_forecasting_demo example  
if cargo run --example basic_forecasting_demo --features forecasting --quiet; then
    echo -e "   ${GREEN}✓ basic_forecasting_demo example works${NC}"
else
    echo -e "   ${RED}✗ basic_forecasting_demo example failed${NC}"
    EXAMPLES_PASSED=false
fi

if $EXAMPLES_PASSED; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    print_status 0 "All examples work correctly"
else
    print_status 1 "Some examples failed"
fi

# 7. Performance Benchmark (if available)
echo -e "\n7. ⚡ Performance Check..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ -f "benches/performance.rs" ]; then
    if cargo bench --features forecasting --quiet; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        print_status 0 "Performance benchmarks passed"
    else
        print_status 1 "Performance benchmarks failed"
    fi
else
    print_warning "No performance benchmarks found"
    # Don't count this as failed since benchmarks are optional
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
fi

# 8. Security Audit (if cargo-audit is available)
echo -e "\n8. 🔒 Security Audit..."
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        print_status 0 "Security audit passed"
    else
        print_status 1 "Security vulnerabilities found"
    fi
else
    print_warning "cargo-audit not found, skipping security check"
    print_warning "Install with: cargo install cargo-audit"
    PASSED_CHECKS=$((PASSED_CHECKS + 1))  # Don't fail for missing tool
fi

# Summary
echo -e "\n📊 Quality Assurance Summary"
echo "============================="
echo "Total Checks: $TOTAL_CHECKS"
echo "Passed: $PASSED_CHECKS"
echo "Failed: $((TOTAL_CHECKS - PASSED_CHECKS))"

SUCCESS_RATE=$((PASSED_CHECKS * 100 / TOTAL_CHECKS))
echo "Success Rate: $SUCCESS_RATE%"

if [ $SUCCESS_RATE -eq 100 ]; then
    echo -e "\n${GREEN}🎉 All quality checks passed! NyxsOwl is production ready.${NC}"
    exit 0
elif [ $SUCCESS_RATE -ge 80 ]; then
    echo -e "\n${YELLOW}⚠ Most quality checks passed, but some issues need attention.${NC}"
    exit 1
else
    echo -e "\n${RED}❌ Multiple quality issues found. Please address them before release.${NC}"
    exit 1
fi 