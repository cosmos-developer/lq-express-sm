#!/bin/bash
source .env

NODE="https://testnet.sentry.tm.injective.network:443"
USER="tuan_inj"
echo "Deploying LQ Express Smart Contract to Injective Chain"
TXID=$(yes $KEYRING_PASSWORD | injectived tx wasm store artifacts/lq_express_sm.wasm --from=$USER --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=$(echo $NODE) --output json | jq -r '.txhash') 
echo "Transaction ID: $TXID"
# Wait for the transaction to be included in a block
echo "Waiting for the transaction to be included in a block"
sleep 10
RES=$(injectived q tx $TXID --node=$(echo $NODE) --output json)
echo "Response: $RES"
# The following is an easier way to get the Code Id from the response:
CODE_ID=$(injectived q tx $TXID --node=$(echo $NODE) --output json \
| jq -r '.logs[0].events[-1].attributes[-2].value' |  tr -d '"' )
# Instantiate the contract
echo "Code ID: $CODE_ID"
echo "Instantiating LQ Express Smart Contract"
# Prepare the instantiation message
INIT='{}'
yes $KEYRING_PASSWORD | injectived tx wasm instantiate $CODE_ID $INIT --from=$USER --chain-id="injective-888" --label="lq_express" --yes --gas-prices=500000000inj --gas=20000000 --node=$(echo $NODE) --admin $(yes $KEYRING_PASSWORD | injectived keys show $USER -a)
# Get the contract address
echo "Getting the contract address"
injectived query wasm list-contract-by-code $CODE_ID --node $NODE --output json
CONTRACT=$(injectived query wasm list-contract-by-code $CODE_ID --node $NODE --output json | jq -r '.contracts[-1]')
echo "Contract address: $CONTRACT"

