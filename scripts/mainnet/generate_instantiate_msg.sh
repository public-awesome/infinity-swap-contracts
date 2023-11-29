CODE_ID=106
MSG=$(cat <<EOF
{
    "fair_burn": "stars1pm9sf7ftn6fmxcsx9cn9cng2r42vfhg7wh58h975cv9tdpcjjrdsgd0lzt",
    "royalty_registry": "stars1whcx239httqpqs83m53p05l7un2u7rf3hewpqczs4a88nzzrf7xs5khzce",
    "marketplace": "stars1fvhcnyddukcqfnt7nlwv3thm5we22lyxyxylr9h77cvgkcn43xfsvgv0pl",
    "pair_creation_fee": {
        "denom": "ustars",
        "amount": "100000000"
    },
    "fair_burn_fee_percent": "0.005",
    "default_royalty_fee_percent": "0.005",
    "max_royalty_fee_percent": "0.05",
    "max_swap_fee_percent": "0.05",
    "min_prices": [{
        "denom": "ustars",
        "amount": "5000000"
    }],
    "code_ids": {
        "infinity_global": 108,
        "infinity_factory": 107,
        "infinity_index": 109,
        "infinity_pair": 110,
        "infinity_router": 111
    },
    "admin": "stars1jwgchjqltama8z0v0smpagmnpjkc8sw8r03xzq"
}
EOF
)

MULTISIG="stars1jwgchjqltama8z0v0smpagmnpjkc8sw8r03xzq"
ADMIN="stars1jwgchjqltama8z0v0smpagmnpjkc8sw8r03xzq"
CHAIN_ID="stargaze-1"
NODE="https://rpc.stargaze-apis.com:443"
SALT=00

starsd tx wasm instantiate2 $CODE_ID "$MSG" $SALT  \
  --label "stargaze-royalty-registry" \
  --from "$MULTISIG" \
  --admin "$ADMIN" \
  --chain-id "$CHAIN_ID" \
  --node "$NODE" \
  --gas-prices 1ustars \
  --gas 5000000 \
  --generate-only \
  > ./unsigned_instantiate_infinity.json
