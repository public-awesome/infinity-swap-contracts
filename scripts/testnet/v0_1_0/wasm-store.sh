#!/bin/bash

# This script is used to download infinity wasm files from github
# and store them on Stargaze testnet.

VERSION=v0.2.1

CONTRACTS=(
    # "infinity_builder"
    "infinity_factory"
    # "infinity_global"
    # "infinity_index"
    "infinity_pair"
    # "infinity_router"
)

FROM="hot-wallet"
CHAIN_ID="elgafar-1"
NODE="https://rpc.elgafar-1.stargaze-apis.com:443"

for contract in "${CONTRACTS[@]}"; do
    contract_tmp=$(echo $contract | tr '_' '-')

    # If file exists then skip
    if [ -f "./artifacts/${contract}.wasm" ]; then
        echo "Skipping ${contract}.wasm"
        continue
    fi

    url=https://github.com/tasiov/infinity-swap/releases/download/${VERSION}/${contract}.wasm
    echo "Downloading $url"
    response_code=$(curl -L --output "./artifacts/${contract}.wasm" --write-out "%{http_code}" "${url}")

    # Check if the download was successful
    if [ "$response_code" -ne 200 ]; then
        echo "Error downloading ${contract}.wasm. HTTP status code: ${response_code}"
        rm "./artifacts/${contract}.wasm"
        exit 1
    fi
done

echo "Uploading wasm files to Stargaze testnet"

for contract in "${CONTRACTS[@]}"; do
    starsd tx wasm store artifacts/${contract}.wasm \
        --from $FROM \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend test \
        --gas-prices 0.1ustars \
        --gas-adjustment 1.7 \
        --gas auto \
        -b block \
        -y
done