#!/bin/bash

# This script is used to unzip the multisig store messages and sign them.

ZIP_FILE="$1"

echo "Unzip wasm store messages..."

mkdir -p tmp
mkdir -p tmp/unsigned
mkdir -p tmp/signed

unzip $ZIP_FILE -d ./tmp/unsigned

FROM=sg-tasio-multisig

# Loop through each filename in the directory
for filepath in ./tmp/unsigned/*.json; do
  echo "Signing $filepath..."
  filename=$(basename "$filepath")
  starsd tx sign $filepath \
    --from $FROM \
    --multisig deploy_multi \
    --chain-id stargaze-1 \
    --keyring-backend os \
    --node https://rpc.stargaze-apis.com:443 \
    > ./tmp/signed/${filename}.json
done

echo "Zipping signed messages..."

cd ./tmp/signed
zip -r signed_store_messages.zip ./*.json
cd ..
