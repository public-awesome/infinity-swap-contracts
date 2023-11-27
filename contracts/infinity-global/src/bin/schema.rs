use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use infinity_global::msg::{InstantiateMsg, QueryMsg, SudoMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: Empty,
        query: QueryMsg,
        sudo: SudoMsg
    }
}
