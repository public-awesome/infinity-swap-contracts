MSG=$(cat <<EOF
{
    "update_config": {
        "infinity_pair_code_id": 3357,
        "pair_creation_fee": {
            "amount": "0",
            "denom": "ustars"
        }
    }
}
EOF
)

INFINITY_GLOBAL_ADDRESS="stars13qt3f46kh2y8t0jrdu89eg79nc9px82q2vvxfljzzudpqu8umydq6hwqrv"

NEW_CODE_ID=3356

FROM="hot-wallet"
CHAIN_ID="elgafar-1"
NODE="https://rpc.elgafar-1.stargaze-apis.com:443"

starsd tx wasm migrate $INFINITY_GLOBAL_ADDRESS $NEW_CODE_ID "$MSG" \
  --from "$FROM" \
  --keyring-backend test \
  --chain-id "$CHAIN_ID" \
  --node "$NODE" \
  --gas-prices 0.1ustars \
  --gas-adjustment 1.7 \
  --gas auto \
  -b block \
  -o json | jq .
