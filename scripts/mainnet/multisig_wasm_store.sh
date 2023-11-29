#!/bin/bash

# This script is used to download infinity wasm files from github
# and generate the multisig store wasm message for deployment on mainnet.

echo "Download royalty registry"

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


echo "Download infinity contracts"

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

MULTISIG="stars1jwgchjqltama8z0v0smpagmnpjkc8sw8r03xzq"
CHAIN_ID="stargaze-1"
NODE="https://rpc.stargaze-apis.com:443"

MULTISIG_INSTANTIATED=(
    "stargaze_royalty_registry"
    "infinity_builder"
)

for contract in "${MULTISIG_INSTANTIATED[@]}"; do
    starsd tx wasm store artifacts/${contract}.wasm \
        --from $MULTISIG \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --gas 5000000 \
        --generate-only \
        --instantiate-anyof-addresses $MULTISIG \
        > ./tmp/unsigned/unsigned_store_${contract}.json 2>&1
done

BUILDER_INSTANTIATED=(
    "infinity_factory"
    "infinity_global"
    "infinity_index"
    "infinity_router"
)

INFINITY_BUILDER=$(
    starsd query wasm build-address \
        a09abb9d5ad824a19eea3a456f10b8999054295cf65f871db94e7b50e0757d48 \
        $MULTISIG \
        00 \
        --chain-id stargaze-1
)

echo "Infinity builder address ${INFINITY_BUILDER}"

for contract in "${BUILDER_INSTANTIATED[@]}"; do
    starsd tx wasm store artifacts/${contract}.wasm \
        --from $MULTISIG \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --gas 5000000 \
        --generate-only \
        --instantiate-anyof-addresses $INFINITY_BUILDER \
        > ./tmp/unsigned/unsigned_store_${contract}.json 2>&1
done

FACTORY_INSTANTIATED=(
    "infinity_pair"
)

INFINITY_FACTORY=$(
    starsd query wasm build-address \
        f296438309413d76179031e0bb0e0c87805317cd72d090f2a94d7a45f0ce731b \
        $INFINITY_BUILDER \
        0e3a3c59a558e61c0152520b7308cd84a1f5225fafe62e87d3925a8eed0de74c \
        --chain-id stargaze-1
)

echo "Infinity factory address ${INFINITY_FACTORY}"

for contract in "${FACTORY_INSTANTIATED[@]}"; do
    starsd tx wasm store artifacts/${contract}.wasm \
        --from $MULTISIG \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --gas 5000000 \
        --generate-only \
        --instantiate-anyof-addresses $INFINITY_FACTORY \
        > ./tmp/unsigned/unsigned_store_${contract}.json 2>&1
done

echo "Zipping wasm store messages..."

cd ./tmp/unsigned
zip -r unsigned_store_messages.zip ./*.json
cd ..
