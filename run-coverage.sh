#!/bin/bash
# Code Coverage Script for SIDS Project
# This script runs code coverage and automatically cleans up artifacts
# All operations stay within the project directory

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Parse arguments
CLEAN=false
HTML=false
JSON=false
LCOV=false
OUTPUT_FORMAT="html"

while [[ $# -gt 0 ]]; do
    case $1 in
        --clean|-c)
            CLEAN=true
            shift
            ;;
        --html|-h)
            HTML=true
            shift
            ;;
        --json|-j)
            JSON=true
            shift
            ;;
        --lcov|-l)
            LCOV=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--clean|-c] [--html|-h] [--json|-j] [--lcov|-l]"
            exit 1
            ;;
    esac
done

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo -e "${CYAN}=== SIDS Code Coverage Runner ===${NC}"
echo ""

# Function to clean up coverage artifacts
clean_coverage_artifacts() {
    echo -e "${YELLOW}Cleaning coverage artifacts...${NC}"
    
    local paths_to_clean=(
        "target/coverage"
        "target/tmp/*.profraw"
        "target/*.profraw"
        "target/*.profdata"
        "coverage"
    )
    
    for path in "${paths_to_clean[@]}"; do
        if [ -e "$path" ] || compgen -G "$path" > /dev/null 2>&1; then
            echo -e "  ${NC}Removing: $path${NC}"
            rm -rf $path 2>/dev/null || true
        fi
    done
    
    echo -e "${GREEN}Coverage artifacts cleaned.${NC}"
    echo ""
}

# If --clean flag is provided, clean and exit
if [ "$CLEAN" = true ]; then
    clean_coverage_artifacts
    exit 0
fi

# Check if cargo-llvm-cov is installed
echo -e "${YELLOW}Checking for cargo-llvm-cov...${NC}"
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo -e "${YELLOW}cargo-llvm-cov not found. Installing...${NC}"
    cargo install cargo-llvm-cov
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to install cargo-llvm-cov${NC}"
        exit 1
    fi
fi

# Clean before running
clean_coverage_artifacts

# Create coverage output directory within project
COVERAGE_DIR="$SCRIPT_DIR/target/coverage"
mkdir -p "$COVERAGE_DIR"

echo -e "${YELLOW}Running code coverage with streaming feature...${NC}"
echo ""

# Determine output format
FORMAT_ARGS=()
if [ "$HTML" = true ]; then
    FORMAT_ARGS+=("--html")
elif [ "$JSON" = true ]; then
    FORMAT_ARGS+=("--json")
elif [ "$LCOV" = true ]; then
    FORMAT_ARGS+=("--lcov")
else
    # Default to HTML
    FORMAT_ARGS+=("--html")
fi

# Run cargo-llvm-cov with proper flags
cargo llvm-cov \
    --features streaming \
    --lib \
    --output-dir "$COVERAGE_DIR" \
    "${FORMAT_ARGS[@]}"

EXIT_CODE=$?

echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}=== Coverage Complete ===${NC}"
    echo ""
    echo -e "${CYAN}Coverage report generated at:${NC}"
    
    if [ "$HTML" = true ] || [ "${FORMAT_ARGS[0]}" = "--html" ]; then
        HTML_REPORT="$COVERAGE_DIR/html/index.html"
        if [ -f "$HTML_REPORT" ]; then
            echo -e "  ${WHITE}HTML: $HTML_REPORT${NC}"
            echo ""
            echo -e "${YELLOW}Opening coverage report in browser...${NC}"
            
            # Try to open in browser (cross-platform)
            if command -v xdg-open &> /dev/null; then
                xdg-open "$HTML_REPORT" &> /dev/null || true
            elif command -v open &> /dev/null; then
                open "$HTML_REPORT" &> /dev/null || true
            elif command -v start &> /dev/null; then
                start "$HTML_REPORT" &> /dev/null || true
            else
                echo -e "${YELLOW}Could not automatically open browser. Please open manually:${NC}"
                echo -e "  ${WHITE}file://$HTML_REPORT${NC}"
            fi
        fi
    elif [ "$JSON" = true ]; then
        echo -e "  ${WHITE}JSON: $COVERAGE_DIR/coverage.json${NC}"
    elif [ "$LCOV" = true ]; then
        echo -e "  ${WHITE}LCOV: $COVERAGE_DIR/lcov.info${NC}"
    fi
    
    echo ""
    echo -e "${YELLOW}To clean up coverage artifacts, run:${NC}"
    echo -e "  ${WHITE}./run-coverage.sh --clean${NC}"
else
    echo -e "${RED}=== Coverage Failed ===${NC}"
    echo -e "${RED}Exit code: $EXIT_CODE${NC}"
    
    # Clean up on failure
    echo ""
    clean_coverage_artifacts
fi

exit $EXIT_CODE
