use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};
use infinity_interface::{NftOrder, SwapParams};
use infinity_macros::infinity_module_query;

/// enum for testing. Important that this derives things / has other
/// attributes so we can be sure we aren't messing with other macros
/// with ours.
#[infinity_module_query]
#[allow(dead_code)]
#[cw_serde]
#[derive(QueryResponses)]
enum Test {
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

    Test::SimSwapTokensForSpecificNfts {
        sender: "sender".to_string(),
        collection: "collection".to_string(),
        nft_orders: nft_orders.clone(),
        swap_params: swap_params.clone(),
    };
    Test::SimSwapTokensForAnyNfts {
        sender: "sender".to_string(),
        collection: "collection".to_string(),
        orders: vec![Uint128::from(10u64)],
        swap_params: swap_params.clone(),
    };
    Test::SimSwapNftsForTokens {
        sender: "sender".to_string(),
        collection: "collection".to_string(),
        nft_orders: nft_orders.clone(),
        swap_params: swap_params.clone(),
    };

    let test = Test::Foo {};

    // If this compiles we have won.
    let result = match test {
        Test::Foo
        | Test::Bar(_)
        | Test::Baz { .. }
        | Test::SimSwapTokensForSpecificNfts { .. }
        | Test::SimSwapTokensForAnyNfts { .. }
        | Test::SimSwapNftsForTokens { .. } => "yay",
    };
    assert_eq!(result, "yay");
}
