# !/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TMP_DIR="$SCRIPT_DIR/tmp"

source "$SCRIPT_DIR/../.env.mainnet"

starsd tx multisign "$TMP_DIR/unsigned_infinity_authz.json" deploy_multi \
    "$TMP_DIR/signed_infinity_authz.stars1zhnkr2t4mdtgng42hfs73qtlcza9zlqhtf5ely.json" \
    "$TMP_DIR/ruwan_signed_infinity_authz.json" \
    "$TMP_DIR/jorge.json" \
    --node $NODE \
    --chain-id $CHAIN_ID \
    > "$TMP_DIR/signed.json"

starsd tx broadcast $TX_OPTIONS "$TMP_DIR/signed.json"