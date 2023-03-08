use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use cosmwasm_std::Uint128;
use infinity_pool::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, NftSwap, PoolNftSwap, PoolQuoteResponse,
    PoolsByIdResponse, PoolsResponse, QueryMsg, QueryOptions, SwapParams, SwapResponse,
};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(NftSwap), &out_dir);
    export_schema(&schema_for!(PoolNftSwap), &out_dir);
    export_schema(&schema_for!(PoolQuoteResponse), &out_dir);
    export_schema(&schema_for!(PoolsByIdResponse), &out_dir);
    export_schema(&schema_for!(PoolsResponse), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(QueryOptions<u64>), &out_dir);
    export_schema(&schema_for!(QueryOptions<(Uint128, u64)>), &out_dir);
    export_schema(&schema_for!(SwapParams), &out_dir);
    export_schema(&schema_for!(SwapResponse), &out_dir);
}
