# !/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
source "$SCRIPT_DIR/../.env.mainnet"

ACCOUNT_JSON=$(starsd q account $MULTISIG --chain-id $CHAIN_ID --node $NODE -o json)

ACCOUNT_NUMBER=$(echo $ACCOUNT_JSON | jq -r .account_number)
SEQUENCE=$(echo $ACCOUNT_JSON | jq -r .sequence)

starsd tx sign "$SCRIPT_DIR/tmp/unsigned_infinity_authz.json" \
    --multisig $GRANTER \
    --from $MY_MULTISIG_ACCOUNT \
    --output-document "$SCRIPT_DIR/tmp/signed_infinity_authz.$MY_MULTISIG_ACCOUNT.json" \
    --sequence $SEQUENCE \
    --account-number $ACCOUNT_NUMBER \
    --chain-id $CHAIN_ID \
    --node $NODE \
    --offline
