#!/bin/bash

# Generate Sample Project Demo - Creates a new sample project with comprehensive test data and launches web UI
# Usage: ./generate-sample-demo.sh [project-name] [port]

set -e

# Configuration
PROJECT_NAME="${1:-sample-project-demo}"
PORT="${2:-3000}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Try to find the built binary, fallback to cargo run
if [ -f "$SCRIPT_DIR/target/debug/wsb" ]; then
    WS_BINARY="$SCRIPT_DIR/target/debug/wsb"
    WS_RUN_CMD="$WS_BINARY"
    BINARY_TYPE="built"
elif [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    WS_BINARY="cargo run --"
    WS_RUN_CMD="cd '$SCRIPT_DIR' && cargo run --"
    BINARY_TYPE="cargo"
else
    echo -e "${RED}❌ Error: Neither built binary nor Cargo.toml found${NC}"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}📊 Sample Project Demo Generator${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "Project Name: ${GREEN}$PROJECT_NAME${NC}"
echo -e "Port: ${GREEN}$PORT${NC}"
echo -e "Binary: ${GREEN}$WS_BINARY${NC} (${BINARY_TYPE})"
echo ""

# Check if port is available
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null ; then
    echo -e "${YELLOW}⚠️  Port $PORT is already in use${NC}"
    echo -e "${YELLOW}💡 Kill existing process or choose a different port${NC}"
    echo -e "${YELLOW}   To kill: sudo kill -9 \$(lsof -ti:$PORT)${NC}"
    exit 1
fi

# Remove existing project directory if it exists
if [ -d "$PROJECT_NAME" ]; then
    echo -e "${YELLOW}🗑️  Removing existing directory: $PROJECT_NAME${NC}"
    rm -rf "$PROJECT_NAME"
fi

echo -e "${BLUE}📁 Creating sample project...${NC}"
# Create the project with comprehensive test data
if [ "$BINARY_TYPE" = "cargo" ]; then
    (cd "$SCRIPT_DIR" && cargo run -- sample --project --data --output "$PROJECT_NAME")
else
    "$WS_BINARY" sample --project --data --output "$PROJECT_NAME"
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to create sample project${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Sample project created successfully!${NC}"
echo ""

# Navigate to project directory
cd "$PROJECT_NAME"

echo -e "${BLUE}📊 Verifying generated data...${NC}"
# Show some stats about what was generated (run from sample project directory)
if [ "$BINARY_TYPE" = "cargo" ]; then
    if [ -f "$SCRIPT_DIR/target/debug/wsb" ]; then
        TASK_COUNT=$("$SCRIPT_DIR/target/debug/wsb" task list 2>/dev/null | grep -E "^  [✅🔄⏳🚫❌]" | wc -l | tr -d ' ')
        FEATURE_OUTPUT=$("$SCRIPT_DIR/target/debug/wsb" feature list 2>/dev/null | grep -o '"count":[0-9]*' | cut -d: -f2 || echo "0")
    else
        TASK_COUNT=$(CARGO_TARGET_DIR="$SCRIPT_DIR/target" cargo run --manifest-path "$SCRIPT_DIR/Cargo.toml" -- task list 2>/dev/null | grep -E "^  [✅🔄⏳🚫❌]" | wc -l | tr -d ' ')
        FEATURE_OUTPUT=$(CARGO_TARGET_DIR="$SCRIPT_DIR/target" cargo run --manifest-path "$SCRIPT_DIR/Cargo.toml" -- feature list 2>/dev/null | grep -o '"count":[0-9]*' | cut -d: -f2 || echo "0")
    fi
else
    TASK_COUNT=$("$WS_BINARY" task list 2>/dev/null | grep -E "^  [✅🔄⏳🚫❌]" | wc -l | tr -d ' ')
    FEATURE_OUTPUT=$("$WS_BINARY" feature list 2>/dev/null | grep -o '"count":[0-9]*' | cut -d: -f2 || echo "0")
fi
echo -e "📋 Tasks generated: ${GREEN}$TASK_COUNT${NC}"
echo -e "⭐ Features generated: ${GREEN}$FEATURE_OUTPUT${NC}"

echo ""
echo -e "${BLUE}🚀 Starting web UI on port $PORT...${NC}"
echo -e "${YELLOW}📱 ADE Interface (Recommended): http://localhost:$PORT/ade${NC}"
echo -e "${YELLOW}📱 Classic Dashboard: http://localhost:$PORT${NC}"
echo -e "${YELLOW}🛑 Press Ctrl+C to stop the server${NC}"
echo ""

# Start the web UI server (run from sample project directory to use its database)
if [ "$BINARY_TYPE" = "cargo" ]; then
    # Try built binary first, fallback to cargo run
    if [ -f "$SCRIPT_DIR/target/debug/wsb" ]; then
        "$SCRIPT_DIR/target/debug/wsb" mcp-server --port "$PORT"
    else
        # Run cargo from the source dir but keep current working directory as sample project
        CARGO_TARGET_DIR="$SCRIPT_DIR/target" cargo run --manifest-path "$SCRIPT_DIR/Cargo.toml" -- mcp-server --port "$PORT"
    fi
else
    "$WS_BINARY" mcp-server --port "$PORT"
fi