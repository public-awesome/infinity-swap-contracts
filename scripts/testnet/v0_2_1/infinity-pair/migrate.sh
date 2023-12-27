#!/bin/bash

# Fetch the list of addresses
ADDRESSES=$(starsd q wasm list-contract-by-code 3357 --limit 10 --output json | jq -r '.contracts[]')

INFINITY_FACTORY_ADDRESS="stars10r4s0uddkuc9r9x5v8ysyhrew223947659jv6gkmvsy29cqv2tuqz3een0"
    
FROM="hot-wallet"
CHAIN_ID="elgafar-1"
NODE="https://rpc.elgafar-1.stargaze-apis.com:443"

# Iterate over each address
for ADDRESS in $ADDRESSES; do
    echo "Processing pair address: $ADDRESS"
    
    MSG=$(cat <<EOF
    {
        "unrestricted_migrate_pair": {
            "pair_address": "$ADDRESS",
            "target_code_id": 3433
        }
    })

    echo "$MSG"

    starsd tx wasm execute $INFINITY_FACTORY_ADDRESS "$MSG" \
        --from "$FROM" \
        --keyring-backend test \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --gas-prices 0.1ustars \
        --gas-adjustment 1.7 \
        --gas auto \
        -b block \
        -o json | jq .
done
