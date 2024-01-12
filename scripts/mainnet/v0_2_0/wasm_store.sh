# !/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TMP_DIR="$SCRIPT_DIR/tmp"

source "$SCRIPT_DIR/../.env.mainnet"

for contract in "${INFINITY_V2_CONTRACTS[@]}"; do
    starsd tx wasm store artifacts/${contract}.wasm --from $GRANTER --generate-only > "$TMP_DIR/tx.${contract}.json" && \
        starsd tx authz exec "$TMP_DIR/tx.${contract}.json" --from $GRANTEE $TX_OPTIONS
done
