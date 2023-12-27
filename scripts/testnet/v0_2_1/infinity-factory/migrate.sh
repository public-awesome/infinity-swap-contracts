MSG=$(cat <<EOF
{
    "add_unrestricted_migration": {
        "starting_code_id": 3357,
        "target_code_id": 3433
    }
}
EOF
)

INFINITY_FACTORY_ADDRESS="stars10r4s0uddkuc9r9x5v8ysyhrew223947659jv6gkmvsy29cqv2tuqz3een0"

NEW_CODE_ID=3432

FROM="hot-wallet"
CHAIN_ID="elgafar-1"
NODE="https://rpc.elgafar-1.stargaze-apis.com:443"

starsd tx wasm migrate $INFINITY_FACTORY_ADDRESS $NEW_CODE_ID "$MSG" \
  --from "$FROM" \
  --keyring-backend test \
  --chain-id "$CHAIN_ID" \
  --node "$NODE" \
  --gas-prices 0.1ustars \
  --gas-adjustment 1.7 \
  --gas auto \
  -b block \
  -o json | jq .
