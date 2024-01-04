# !/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TMP_DIR="$SCRIPT_DIR/../tmp"

source "$SCRIPT_DIR/../../.env.mainnet"

CONTRACT="infinity_router"

GLOBAL_CONFIG=$(starsd q wasm contract-state smart $INFINITY_GLOBAL '{"global_config":{}}' --chain-id $CHAIN_ID --node $NODE -o json)
INFINITY_ROUTER=$(echo $GLOBAL_CONFIG | jq -r ".data.$CONTRACT")

starsd tx wasm migrate $INFINITY_ROUTER $INFINITY_ROUTER_V2_CODE_ID "{}" \
    --from $GRANTER \
    --generate-only \
    > "$TMP_DIR/migrate_tx.$CONTRACT.json"

starsd tx authz exec "$TMP_DIR/migrate_tx.$CONTRACT.json" \
    --from $GRANTEE \
    $TX_OPTIONS

