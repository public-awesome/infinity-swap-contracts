# !/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TMP_DIR="$SCRIPT_DIR/../tmp"

source "$SCRIPT_DIR/../../.env.mainnet"

CONTRACT="infinity_factory"

GLOBAL_CONFIG=$(starsd q wasm contract-state smart $INFINITY_GLOBAL '{"global_config":{}}' --chain-id $CHAIN_ID --node $NODE -o json)
INFINITY_FACTORY=$(echo $GLOBAL_CONFIG | jq -r ".data.$CONTRACT")

MSG=$(cat <<EOF
{
    "add_unrestricted_migration": {
        "starting_code_id": $INFINITY_PAIR_V1_CODE_ID,
        "target_code_id": $INFINITY_PAIR_V2_CODE_ID
    }
}
EOF
)

starsd tx wasm migrate $INFINITY_FACTORY $INFINITY_FACTORY_V2_CODE_ID "$MSG" \
    --from $GRANTER \
    --generate-only \
    > "$TMP_DIR/migrate_tx.$CONTRACT.json"

starsd tx authz exec "$TMP_DIR/migrate_tx.$CONTRACT.json" \
    --from $GRANTEE \
    $TX_OPTIONS

