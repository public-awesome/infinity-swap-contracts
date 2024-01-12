#!/bin/bash

# !/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TMP_DIR="$SCRIPT_DIR/../tmp"

source "$SCRIPT_DIR/../../.env.mainnet"

GLOBAL_CONFIG=$(starsd q wasm contract-state smart $INFINITY_GLOBAL '{"global_config":{}}' --chain-id $CHAIN_ID --node $NODE -o json)
INFINITY_FACTORY=$(echo $GLOBAL_CONFIG | jq -r ".data.infinity_factory")

ADDRESSES=$(starsd q wasm list-contract-by-code $INFINITY_PAIR_V1_CODE_ID --limit 200 --chain-id $CHAIN_ID --node $NODE --output json | jq -r '.contracts[]')

# Iterate over each address
COUNTER=0
for ADDRESS in $ADDRESSES; do
    let COUNTER=COUNTER+1

    echo "Processing pair #$COUNTER address: $ADDRESS"
    
    MSG=$(cat <<EOF
    {
        "unrestricted_migrate_pair": {
            "pair_address": "$ADDRESS",
            "target_code_id": $INFINITY_PAIR_V2_CODE_ID
        }
    })

    starsd tx wasm execute $INFINITY_FACTORY "$MSG" \
        --from "$GRANTEE" \
        $TX_OPTIONS \
        -y

    sleep 1
done
