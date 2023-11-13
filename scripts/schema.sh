#!/bin/bash

start_dir=$(pwd)

rm -rf schema

for contract_path in contracts/*; do
  if [ -d "$contract_path" ]; then
    cd $contract_path
    filename="$(basename $contract_path)"
    cargo schema
    rm -rf schema/raw
    mkdir -p $start_dir/schema/$filename
    mv schema/$filename.json $start_dir/schema/$filename/$filename.json
    cd "$start_dir"
  fi
done