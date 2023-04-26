use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};
use infinity_macros::{infinity_module_execute, infinity_module_query};
use infinity_shared::interface::{NftOrder, SwapParams};

#[infinity_module_query]
#[allow(dead_code)]
#[cw_serde]
#[derive(QueryResponses)]
enum TestQueryMsg {
    #[returns(String)]
    Foo,
    #[returns(String)]
    Bar(u64),
    #[returns(String)]
    Baz { waldo: u64 },
}

#[test]
fn infinity_module_query_derive() {
    let nft_orders: Vec<NftOrder> = vec![NftOrder {
        token_id: "1".to_string(),
        amount: Uint128::from(10u64),
    }];

    let swap_params = SwapParams {
        deadline: Timestamp::from_seconds(10),
        robust: true,
        asset_recipient: Some("asset_recipient".to_string()),
        finder: Some("finder".to_string()),
    };
    TestQueryMsg::SimSwapTokensForAnyNfts {
        sender: "sender".to_string(),
        collection: "collection".to_string(),
        orders: vec![Uint128::from(10u64)],
        swap_params: swap_params.clone(),
    };
    TestQueryMsg::SimSwapNftsForTokens {
        sender: "sender".to_string(),
        collection: "collection".to_string(),
        nft_orders: nft_orders.clone(),
        swap_params: swap_params.clone(),
    };

    let test = TestQueryMsg::Foo {};

    // If this compiles we have won.
    let result = match test {
        TestQueryMsg::Foo
        | TestQueryMsg::Bar(_)
        | TestQueryMsg::Baz { .. }
        | TestQueryMsg::SimSwapTokensForAnyNfts { .. }
        | TestQueryMsg::SimSwapNftsForTokens { .. } => "yay",
    };
    assert_eq!(result, "yay");
}

#[infinity_module_execute]
#[allow(dead_code)]
#[cw_serde]
enum TestExecuteMsg {
    Foo,
    Bar(u64),
    Baz { waldo: u64 },
}

#[test]
fn infinity_module_execute_derive() {
    let nft_orders: Vec<NftOrder> = vec![NftOrder {
        token_id: "1".to_string(),
        amount: Uint128::from(10u64),
    }];

    let swap_params = SwapParams {
        deadline: Timestamp::from_seconds(10),
        robust: true,
        asset_recipient: Some("asset_recipient".to_string()),
        finder: Some("finder".to_string()),
    };
    TestExecuteMsg::SwapTokensForAnyNfts {
        collection: "collection".to_string(),
        orders: vec![Uint128::from(10u64)],
        swap_params: swap_params.clone(),
    };
    TestExecuteMsg::SwapNftsForTokens {
        collection: "collection".to_string(),
        nft_orders: nft_orders.clone(),
        swap_params: swap_params.clone(),
    };

    let test = TestExecuteMsg::Foo {};

    // If this compiles we have won.
    let result = match test {
        TestExecuteMsg::Foo
        | TestExecuteMsg::Bar(_)
        | TestExecuteMsg::Baz { .. }
        | TestExecuteMsg::SwapTokensForAnyNfts { .. }
        | TestExecuteMsg::SwapNftsForTokens { .. } => "yay",
    };
    assert_eq!(result, "yay");
}
