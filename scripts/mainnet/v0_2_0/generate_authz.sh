#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
source "$SCRIPT_DIR/../.env.mainnet"

EXPIRATION=1704394256

starsd tx authz grant $GRANTEE generic \
        --from $GRANTER \
        --expiration $EXPIRATION \
        --generate-only \
        $TX_OPTIONS \
        > "$SCRIPT_DIR/tmp/unsigned_infinity_authz.json"