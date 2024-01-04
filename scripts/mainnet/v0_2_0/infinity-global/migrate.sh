# !/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TMP_DIR="$SCRIPT_DIR/../tmp"

source "$SCRIPT_DIR/../../.env.mainnet"

CONTRACT="infinity_global"

MSG=$(cat <<EOF
{
    "update_config": {
        "infinity_pair_code_id": $INFINITY_PAIR_V2_CODE_ID,
        "pair_creation_fee": {
            "amount": "0",
            "denom": "ustars"
        }
    }
}
EOF
)

starsd tx wasm migrate $INFINITY_GLOBAL $INFINITY_GLOBAL_V2_CODE_ID "$MSG" \
    --from $GRANTER \
    --generate-only \
    > "$TMP_DIR/migrate_tx.$CONTRACT.json"

starsd tx authz exec "$TMP_DIR/migrate_tx.$CONTRACT.json" \
    --from $GRANTEE \
    $TX_OPTIONS
