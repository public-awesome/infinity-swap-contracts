#!/bin/bash

# This script is used to download marketplace wasm files from github
# and place them in the proper location for the integration tests.

MARKETPLACE_VERSION="v1.2.0"

contracts=(
    "sg_marketplace"
)

for contract in "${contracts[@]}"; do
    echo "Downloading $contract"
    curl -L --output "./artifacts/${contract}.wasm" "https://github.com/public-awesome/marketplace/releases/download/${MARKETPLACE_VERSION}/${contract}.wasm"
done