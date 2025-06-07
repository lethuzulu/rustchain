#!/bin/bash

# RustChain Multi-Node Test Setup Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🛠️  RustChain Multi-Node Test Setup${NC}"

# Function to cleanup processes
cleanup() {
    echo -e "${YELLOW}Cleaning up background processes...${NC}"
    pkill -f rustchain || true
    sleep 2
}

# Function to setup test databases
setup_dbs() {
    echo -e "${GREEN}Setting up test databases...${NC}"
    rm -rf "$PROJECT_ROOT"/test_node_*_db
    mkdir -p "$PROJECT_ROOT"/test_node_{1,2,3}_db
}

# Function to create node configs
create_configs() {
    echo -e "${GREEN}Creating node configurations...${NC}"
    
    # Node 1 config
    cat > "$SCRIPT_DIR/node1-config.toml" << EOF
genesis_file = "dev/test_genesis.json"

[network]
listen_port = 9001
listen_addr = "127.0.0.1"
bootstrap_peers = []
max_peers = 50

[storage]
db_path = "test_node_1_db"
create_if_missing = true

[consensus]
block_interval = 3
max_txs_per_block = 5

[validator]
enabled = true
private_key_path = "dev/node1-validator.key"
EOF

    # Node 2 config
    cat > "$SCRIPT_DIR/node2-config.toml" << EOF
genesis_file = "dev/test_genesis.json"

[network]
listen_port = 9002
listen_addr = "127.0.0.1"
bootstrap_peers = ["/ip4/127.0.0.1/tcp/9001"]
max_peers = 50

[storage]
db_path = "test_node_2_db"
create_if_missing = true

[consensus]
block_interval = 3
max_txs_per_block = 5
EOF

    # Node 3 config
    cat > "$SCRIPT_DIR/node3-config.toml" << EOF
genesis_file = "dev/test_genesis.json"

[network]
listen_port = 9003
listen_addr = "127.0.0.1"
bootstrap_peers = ["/ip4/127.0.0.1/tcp/9001", "/ip4/127.0.0.1/tcp/9002"]
max_peers = 50

[storage]
db_path = "test_node_3_db"
create_if_missing = true

[consensus]
block_interval = 3
max_txs_per_block = 5
EOF
}

# Function to start nodes
start_nodes() {
    echo -e "${GREEN}Starting test nodes...${NC}"
    
    cd "$PROJECT_ROOT"
    
    echo -e "${YELLOW}Starting Node 1 (Validator)...${NC}"
    cargo run --bin rustchain -- node --config dev/node1-config.toml > dev/node1.log 2>&1 &
    NODE1_PID=$!
    echo "Node 1 PID: $NODE1_PID"
    
    sleep 3
    
    echo -e "${YELLOW}Starting Node 2...${NC}"
    cargo run --bin rustchain -- node --config dev/node2-config.toml > dev/node2.log 2>&1 &
    NODE2_PID=$!
    echo "Node 2 PID: $NODE2_PID"
    
    sleep 3
    
    echo -e "${YELLOW}Starting Node 3...${NC}"
    cargo run --bin rustchain -- node --config dev/node3-config.toml > dev/node3.log 2>&1 &
    NODE3_PID=$!
    echo "Node 3 PID: $NODE3_PID"
    
    echo -e "${GREEN}All nodes started. PIDs: $NODE1_PID, $NODE2_PID, $NODE3_PID${NC}"
    echo "Logs available in: dev/node{1,2,3}.log"
    echo ""
    echo "To stop all nodes: pkill -f rustchain"
    echo "To view logs: tail -f dev/node1.log"
}

# Function to stop nodes
stop_nodes() {
    echo -e "${YELLOW}Stopping all test nodes...${NC}"
    cleanup
    echo -e "${GREEN}All nodes stopped.${NC}"
}

# Function to show status
show_status() {
    echo -e "${GREEN}Node Status:${NC}"
    if pgrep -f "rustchain.*node1-config" > /dev/null; then
        echo -e "Node 1: ${GREEN}RUNNING${NC}"
    else
        echo -e "Node 1: ${RED}STOPPED${NC}"
    fi
    
    if pgrep -f "rustchain.*node2-config" > /dev/null; then
        echo -e "Node 2: ${GREEN}RUNNING${NC}"
    else
        echo -e "Node 2: ${RED}STOPPED${NC}"
    fi
    
    if pgrep -f "rustchain.*node3-config" > /dev/null; then
        echo -e "Node 3: ${GREEN}RUNNING${NC}"
    else
        echo -e "Node 3: ${RED}STOPPED${NC}"
    fi
}

# Function to tail logs
tail_logs() {
    echo -e "${GREEN}Tailing logs (Ctrl+C to stop)...${NC}"
    tail -f "$PROJECT_ROOT"/dev/node*.log
}

# Main command handler
case "${1:-start}" in
    "setup")
        cleanup
        setup_dbs
        create_configs
        echo -e "${GREEN}✅ Test environment set up. Run '$0 start' to start nodes.${NC}"
        ;;
    "start")
        setup_dbs
        create_configs
        start_nodes
        ;;
    "stop")
        stop_nodes
        ;;
    "status")
        show_status
        ;;
    "logs")
        tail_logs
        ;;
    "clean")
        cleanup
        rm -rf "$PROJECT_ROOT"/test_node_*_db
        rm -f "$SCRIPT_DIR"/node*-config.toml
        rm -f "$PROJECT_ROOT"/dev/node*.log
        echo -e "${GREEN}✅ Cleaned up all test files.${NC}"
        ;;
    *)
        echo "Usage: $0 {setup|start|stop|status|logs|clean}"
        echo ""
        echo "Commands:"
        echo "  setup  - Set up test environment"
        echo "  start  - Start all test nodes"
        echo "  stop   - Stop all test nodes"
        echo "  status - Show node status"
        echo "  logs   - Tail all node logs"
        echo "  clean  - Clean up all test files"
        exit 1
        ;;
esac 