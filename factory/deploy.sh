#!/bin/sh

echo ">> Deploying contract"

# https://docs.near.org/tools/near-cli#near-dev-deploy
near dev-deploy --wasmFile ../target/wasm32-unknown-unknown/release/chershare_factory.wasm -f # for new contract id
