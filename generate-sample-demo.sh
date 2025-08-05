#!/bin/bash

# Generate Sample Project Demo - Creates a new sample project with comprehensive test data and launches web UI
# Usage: ./generate-sample-demo.sh [project-name] [port]

set -e

# Configuration
PROJECT_NAME="${1:-sample-project-demo}"
PORT="${2:-3000}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WS_BINARY="$SCRIPT_DIR/target/debug/ws"

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
echo -e "Binary: ${GREEN}$WS_BINARY${NC}"
echo ""

# Check if binary exists
if [ ! -f "$WS_BINARY" ]; then
    echo -e "${RED}❌ Error: ws binary not found at $WS_BINARY${NC}"
    echo -e "${YELLOW}💡 Run 'cargo build' first to create the binary${NC}"
    exit 1
fi

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
"$WS_BINARY" sample --project --data --output "$PROJECT_NAME"

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to create sample project${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Sample project created successfully!${NC}"
echo ""

# Navigate to project directory
cd "$PROJECT_NAME"

echo -e "${BLUE}📊 Verifying generated data...${NC}"
# Show some stats about what was generated
TASK_COUNT=$("$SCRIPT_DIR/target/debug/ws" task list 2>/dev/null | grep -E "^  [✅🔄⏳🚫❌]" | wc -l | tr -d ' ')
echo -e "📋 Tasks generated: ${GREEN}$TASK_COUNT${NC}"

# Check if we have any features (the API might not be working yet)
FEATURE_OUTPUT=$("$SCRIPT_DIR/target/debug/ws" feature list 2>/dev/null | grep -o '"count":[0-9]*' | cut -d: -f2 || echo "0")
echo -e "⭐ Features generated: ${GREEN}$FEATURE_OUTPUT${NC}"

echo ""
echo -e "${BLUE}🚀 Starting web UI on port $PORT...${NC}"
echo -e "${YELLOW}📱 Dashboard will be available at: http://localhost:$PORT${NC}"
echo -e "${YELLOW}🛑 Press Ctrl+C to stop the server${NC}"
echo ""

# Start the web UI server
"$SCRIPT_DIR/target/debug/ws" mcp-server --port "$PORT"