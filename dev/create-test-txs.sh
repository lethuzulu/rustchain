#!/bin/bash

# RustChain Test Transaction Creator

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🧪 RustChain Test Transaction Creator${NC}"

# Test wallet addresses (from our test genesis)
VALIDATOR_ADDR="d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
TEST_ADDR_1="1111111111111111111111111111111111111111111111111111111111111111"
TEST_ADDR_2="2222222222222222222222222222222222222222222222222222222222222222"
TEST_ADDR_3="3333333333333333333333333333333333333333333333333333333333333333"

create_wallet_if_not_exists() {
    if [ ! -f "test_wallet.key" ]; then
        echo -e "${YELLOW}Creating test wallet...${NC}"
        cd "$PROJECT_ROOT"
        cargo run -- wallet generate --output test_wallet.key
    fi
}

# Function to create valid transaction
create_valid_tx() {
    echo -e "${GREEN}Creating valid transaction...${NC}"
    cd "$PROJECT_ROOT"
    
    # Create a simple transfer from test account 1 to test account 2
    create_wallet_if_not_exists
    
    echo "Creating transaction: 1000 tokens from $TEST_ADDR_1 to $TEST_ADDR_2"
    # Note: This is a placeholder - we'll need to implement proper wallet functionality
    echo "Command: cargo run -- wallet send --to $TEST_ADDR_2 --amount 1000"
}

# Function to create invalid transactions for testing
create_invalid_txs() {
    echo -e "${YELLOW}Creating invalid transaction scenarios...${NC}"
    
    echo "1. Insufficient balance transaction"
    echo "   - Send 999999999 tokens (more than available)"
    
    echo "2. Invalid nonce transaction"
    echo "   - Send with wrong nonce value"
    
    echo "3. Invalid signature transaction"
    echo "   - Manually tamper with signature"
    
    echo "4. Malformed transaction data"
    echo "   - Send malformed serialized data"
    
    echo "5. Send to non-existent address"
    echo "   - Send to random address (should create account)"
}

# Function to test double spending
create_double_spend() {
    echo -e "${RED}Creating double spend scenario...${NC}"
    
    echo "1. Create Transaction A: Send 500 tokens to Address X"
    echo "2. Create Transaction B: Send same 500 tokens to Address Y" 
    echo "3. Submit both transactions to different nodes simultaneously"
    echo "4. Observe which transaction gets included in block"
}

# Function to create stress test transactions
create_stress_test() {
    echo -e "${YELLOW}Creating stress test transactions...${NC}"
    
    echo "Creating multiple transactions to fill mempool..."
    echo "This will test mempool capacity and block building logic"
}

# Show menu
show_menu() {
    echo ""
    echo "Available test scenarios:"
    echo "1. valid      - Create valid transaction"
    echo "2. invalid    - Show invalid transaction scenarios"
    echo "3. double     - Create double spend scenario"
    echo "4. stress     - Create stress test transactions"
    echo "5. help       - Show this menu"
}

# Main command handler
case "${1:-help}" in
    "valid")
        create_valid_tx
        ;;
    "invalid")
        create_invalid_txs
        ;;
    "double")
        create_double_spend
        ;;
    "stress")
        create_stress_test
        ;;
    "help"|*)
        show_menu
        ;;
esac

echo ""
echo -e "${GREEN}Note: Some transaction tests require proper wallet implementation${NC}"
echo -e "${GREEN}These scenarios are for manual testing guidance${NC}" 