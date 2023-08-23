MSG=$(cat <<EOF
{
    "fair_burn": "stars177jd3r8aul2dgt9pj77x8zem3au46ee2cj4srxwqdw4lkpd7tsqquz2r2d",
    "royalty_registry": "stars1crgx0f70fzksa57hq87wtl8f04h0qyk5la0hk0fu8dyhl67ju80qaxzr5z",
    "marketplace": "stars18cszlvm6pze0x9sz32qnjq4vtd45xehqs8dq7cwy8yhq35wfnn3qgzs5gu",
    "pair_creation_fee": {
        "denom": "ustars",
        "amount": "1000000"
    },
    "fair_burn_fee_percent": "0.01",
    "max_royalty_fee_percent": "0.05",
    "max_swap_fee_percent": "0.1",
    "code_ids": {
        "infinity_factory": 2894,
        "infinity_global": 2895,
        "infinity_index": 2896,
        "infinity_pair": 2897,
        "infinity_router": 2898
    },
    "min_prices":[
        {
            "denom": "ustars",
            "amount": "1000000"
        }
    ]
}

EOF
)

CODE_ID=2893

LABEL="Infinity Builder"

ADMIN="stars19mmkdpvem2xvrddt8nukf5kfpjwfslrsu7ugt5"
FROM="hot-wallet"

CHAIN_ID="elgafar-1"
NODE="https://rpc.elgafar-1.stargaze-apis.com:443"

starsd tx wasm instantiate $CODE_ID "$MSG" \
  --label "$LABEL" \
  --admin "$ADMIN" \
  --from "$FROM" \
  --keyring-backend test \
  --chain-id "$CHAIN_ID" \
  --node "$NODE" \
  --gas-prices 0.1ustars \
  --gas-adjustment 1.7 \
  --gas auto \
  -b block \
  -o json | jq .
