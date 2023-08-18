use cosmwasm_schema::{cw_serde, write_api, QueryResponses};
use cosmwasm_std::Empty;
use infinity_builder::InstantiateMsg;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: Empty,
        query: QueryMsg,
    }
}
