#!/bin/bash

# This script is used to download infinity wasm files from github
# and generate the multisig store wasm message for deployment on mainnet.

OUT_DIR=$1

cd $OUT_DIR

VERSION=v0.3.0
CONTRACT=stargaze_royalty_registry

CORE_CONTRACTS=(
    "stargaze_royalty_registry"
)

for contract in "${CORE_CONTRACTS[@]}"; do
    contract_tmp=$(echo $contract | tr '_' '-')

    version=${VERSION}
    url=https://github.com/public-awesome/core/releases/download/${contract_tmp}/${version}/${contract}.wasm
    echo "Downloading $url"
    response_code=$(curl -L --output "./${contract}.wasm" --write-out "%{http_code}" "${url}")

    # Check if the download was successful
    if [ "$response_code" -ne 200 ]; then
        echo "Error downloading ${contract}.wasm. HTTP status code: ${response_code}"
        rm "./${contract}.wasm"
        exit 1
    fi
done


VERSION=v0.1.6
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

    url=https://github.com/tasiov/infinity-swap/releases/download/${VERSION}/${contract}.wasm
    echo "Downloading $url"
    response_code=$(curl -L --output "./${contract}.wasm" --write-out "%{http_code}" "${url}")

    # Check if the download was successful
    if [ "$response_code" -ne 200 ]; then
        echo "Error downloading ${contract}.wasm. HTTP status code: ${response_code}"
        rm "./${contract}.wasm"
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
    starsd tx wasm store ./${contract}.wasm \
        --from $MULTISIG \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --gas 5000000 \
        --generate-only \
        --instantiate-anyof-addresses $MULTISIG \
        > ./unsigned_store_${contract}.json
done

BUILDER_INSTANTIATED=(
    "infinity_factory"
    "infinity_global"
    "infinity_index"
    "infinity_router"
)

INFINITY_BUILDER_CHECKSUM=272739b1e2557a6223464f1113471afc6cecc909488efce1b1f1859779b3e135
INFINITY_BUILDER_SALT=00

INFINITY_BUILDER=$(
    starsd query wasm build-address \
        $INFINITY_BUILDER_CHECKSUM \
        $MULTISIG \
        $INFINITY_BUILDER_SALT \
        --chain-id stargaze-1
)

echo "Infinity builder address ${INFINITY_BUILDER}"

for contract in "${BUILDER_INSTANTIATED[@]}"; do
    starsd tx wasm store ./${contract}.wasm \
        --from $MULTISIG \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --gas 5000000 \
        --generate-only \
        --instantiate-anyof-addresses $INFINITY_BUILDER \
        > ./unsigned_store_${contract}.json
done

FACTORY_INSTANTIATED=(
    "infinity_pair"
)

INFINITY_FACTORY_CHECKSUM=7909648e49d22932969dee394fcd89bf3f9c5cd9baf768f0f1aa226e64638975
INFINITY_FACTORY_SALT=0e3a3c59a558e61c0152520b7308cd84a1f5225fafe62e87d3925a8eed0de74c

INFINITY_FACTORY=$(
    starsd query wasm build-address \
        $INFINITY_FACTORY_CHECKSUM \
        $INFINITY_BUILDER \
        $INFINITY_FACTORY_SALT \
        --chain-id stargaze-1
)

echo "Infinity factory address ${INFINITY_FACTORY}"

for contract in "${FACTORY_INSTANTIATED[@]}"; do
    starsd tx wasm store ./${contract}.wasm \
        --from $MULTISIG \
        --chain-id "$CHAIN_ID" \
        --node "$NODE" \
        --keyring-backend os \
        --gas-prices 1ustars \
        --gas 5000000 \
        --generate-only \
        --instantiate-anyof-addresses $INFINITY_FACTORY \
        > ./unsigned_store_${contract}.json
done

echo "Zipping wasm store messages..."

zip -r unsigned_store_messages.zip ./*.json
rm ./*.json
rm ./*.wasm

cd -
