use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use infinity_global::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: Empty,
        query: QueryMsg,
    }
}
