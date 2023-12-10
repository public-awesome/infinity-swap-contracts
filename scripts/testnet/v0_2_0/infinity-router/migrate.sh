INFINITY_ROUTER_ADDRESS="stars1shh4q5kea9x2swsp6nhf0pxlksm5qrdm9p2attcsudmdcdnm2gasth2mg2"

NEW_CODE_ID=3358

FROM="hot-wallet"
CHAIN_ID="elgafar-1"
NODE="https://rpc.elgafar-1.stargaze-apis.com:443"

starsd tx wasm migrate $INFINITY_ROUTER_ADDRESS $NEW_CODE_ID "{}" \
  --from "$FROM" \
  --keyring-backend test \
  --chain-id "$CHAIN_ID" \
  --node "$NODE" \
  --gas-prices 0.1ustars \
  --gas-adjustment 1.7 \
  --gas auto \
  -b block \
  -o json | jq .
