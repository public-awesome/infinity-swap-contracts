#!/bin/bash

# This script is used to download launchpad wasm files from github
# and place them in the proper location for the integration tests.

LAUNCHPAD_VERSION="v2.3.0"

contracts=(
    "base_factory"
    "base_minter"
    "sg721_base"
    # "sg721_metadata_onchain"
    # "sg721_nt"
    # "sg_eth_airdrop"
    # "sg_splits"
    # "sg_whitelist"
    # "vending_factory"
    # "vending_minter"
    # "whitelist_immutable"
)

for contract in "${contracts[@]}"; do
    echo "Downloading $contract"
    curl -L --output "./artifacts/${contract}.wasm" "https://github.com/public-awesome/launchpad/releases/download/${LAUNCHPAD_VERSION}/${contract}.wasm"
done
