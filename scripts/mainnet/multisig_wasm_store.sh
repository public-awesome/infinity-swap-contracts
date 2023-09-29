#!/bin/bash

# This script is used to download infinity wasm files from github
# and generate the multisig store wasm message for deployment on mainnet.

# Download royalty registry

VERSION=v0.3.0
CONTRACT=stargaze_royalty_registry

CORE_CONTRACTS=(
    "stargaze_royalty_registry"
)

for contract in "${CORE_CONTRACTS[@]}"; do
    contract_tmp=$(echo $contract | tr '_' '-')

    # If file exists then skip
    if [ -f "./artifacts/${contract}.wasm" ]; then
        echo "Skipping ${contract}.wasm"
        continue
    fi

    version=${VERSION}
    url=https://github.com/public-awesome/core/releases/download/${contract_tmp}/${version}/${contract}.wasm
    
    if [ -z "$version" ]; then
        echo "Version not found for $contract"
        exit 1
    fi

    echo "Downloading $url"
    curl -L --output "./artifacts/${contract}.wasm" "${url}"
done


Download infinity contracts

VERSION=v0.1.5
INFINITY_CONTRACTS=(
    "infinity_builder"
    "infinity_factory"
    "infinity_global"
    "infinity_index"
    "infinity_pair"
    "infinity_router"
)

for contract in "${INFINITY_CONTRACTS[@]}"; do
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

echo "Generating wasm store messages for mainnet..."

FROM="stars1jwgchjqltama8z0v0smpagmnpjkc8sw8r03xzq"
CHAIN_ID="stargaze-1"
NODE="https://rpc.stargaze-apis.com:443"

for contract in "${CORE_CONTRACTS[@]}"; do
    starsd tx wasm store artifacts/${contract}.wasm \
        --from $FROM \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --generate-only \
        > ./tmp/unsigned_store_${contract}.json 2>&1
done

for contract in "${INFINITY_CONTRACTS[@]}"; do
    starsd tx wasm store artifacts/${contract}.wasm \
        --from $FROM \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --generate-only \
        > ./tmp/unsigned_store_${contract}.json 2>&1
done