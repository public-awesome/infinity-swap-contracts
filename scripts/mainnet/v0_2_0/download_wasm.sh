#!/bin/bash

echo "Download infinity contracts"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
source "$SCRIPT_DIR/../.env.mainnet"

VERSION=v0.2.1

for contract in "${INFINITY_V2_CONTRACTS[@]}"; do
    contract_tmp=$(echo $contract | tr '_' '-')

    # If file exists then skip
    if [ -f "./artifacts/${contract}.wasm" ]; then
        echo "Skipping ${contract}.wasm"
        continue
    fi

    url=https://github.com/public-awesome/infinity-swap-contracts/releases/download/${VERSION}/${contract}.wasm
    echo "Downloading $url"
    response_code=$(curl -L --output "./artifacts/${contract}.wasm" --write-out "%{http_code}" "${url}")

    # Check if the download was successful
    if [ "$response_code" -ne 200 ]; then
        echo "Error downloading ${contract}.wasm. HTTP status code: ${response_code}"
        rm "./artifacts/${contract}.wasm"
        exit 1
    fi
done

