CONTRACTS=(
    "stargaze_royalty_registry"
    "infinity_builder"
    "infinity_factory"
    "infinity_global"
    "infinity_index"
    "infinity_pair"
    "infinity_router"
)
MULTISIG=stars1jwgchjqltama8z0v0smpagmnpjkc8sw8r03xzq
KEY_NAME="sg-tasio-multisig"

CHAIN_ID="stargaze-1"
NODE="https://rpc.stargaze-apis.com:443"

START_SEQ=$(starsd q account $MULTISIG --chain-id $CHAIN_ID --node $NODE --output json | jq -r '.sequence')
ACCOUNT_NUMBER=$(starsd q account $MULTISIG --chain-id $CHAIN_ID --node $NODE --output json | jq '.account_number | tonumber')

ZIP_FILE="$1"
OUT_DIR="$2"

unzip $ZIP_FILE -d $OUT_DIR

cd $OUT_DIR

for i in ${!CONTRACTS[@]}
do
    SEQ=$((START_SEQ + i))
    CONTRACT=${CONTRACTS[$i]}

    echo "Signing $CONTRACT..."
    starsd tx sign ./unsigned_store_$CONTRACT.json \
        --multisig $MULTISIG \
        --from $KEY_NAME \
        --output-document ./signed_store_$CONTRACT.$KEY_NAME.json \
        --sequence $SEQ \
        --account-number $ACCOUNT_NUMBER \
        --chain-id $CHAIN_ID \
        --node $NODE \
        --offline
done

zip -r signed_store_messages.zip ./signed_store_*.json
rm *.json

cd -
