use crate::msg::{PoolQuoteResponse, QueryMsg, QueryOptions, SwapResponse};
use crate::state::{PoolQuote, PoolType};
use crate::testing::setup::setup_accounts::setup_second_bidder_account;
use crate::testing::setup::templates::standard_minter_template;
use crate::testing::tests::sim_tests::helpers::{
    check_nft_sale, deposit_nfts_and_tokens, get_sim_swap_nfts_for_tokens_msg, set_pool_active,
    setup_swap_pool, DepositNftsResult, SwapPoolResult, SwapPoolSetup, VendingTemplateSetup,
};
use cosmwasm_std::Uint128;
use cosmwasm_std::{Addr, StdResult};
use std::vec;

#[test]
fn can_swap_two_active_pools() {
    let spot_price_1 = 1000_u128;
    let spot_price_2 = 1200_u128;
    let vt = standard_minter_template(5000);

    let mut router = vt.router;
    let collection = vt.collection_response_vec[0].collection.as_ref().unwrap();
    let minter = vt.collection_response_vec[0].minter.as_ref().unwrap();
    let user2 = setup_second_bidder_account(&mut router).unwrap();
    let creator = vt.accts.creator;

    let vts = VendingTemplateSetup {
        router: &mut router,
        minter: vt.collection_response_vec[0].minter.as_ref().unwrap(),
        creator: creator.clone(),
        user1: vt.accts.bidder,
        user2: user2.clone(),
        collection: &collection.clone(),
    };
    let swap_pool_configs = vec![
        SwapPoolSetup {
            pool_type: PoolType::Trade,
            spot_price: spot_price_1,
            finders_fee_bps: None,
        },
        SwapPoolSetup {
            pool_type: PoolType::Trade,
            spot_price: spot_price_2,
            finders_fee_bps: None,
        },
    ];
    let swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(vts, swap_pool_configs, None);

    let sprs: Vec<Result<SwapPoolResult, anyhow::Error>> = swap_results;
    let infinity_pool = &sprs[1].as_ref().unwrap().infinity_pool.clone();
    let mut token_id_1 = 0;

    for spr in sprs {
        let spr = spr.unwrap();
        set_pool_active(
            &mut router,
            true,
            spr.pool.clone(),
            creator.clone(),
            infinity_pool.clone(),
        );

        let dnr: DepositNftsResult = deposit_nfts_and_tokens(
            &mut router,
            spr.user1,
            2000_u128,
            Addr::unchecked(minter.to_string()),
            Addr::unchecked(collection.to_string()),
            infinity_pool.clone(),
            spr.pool.clone(),
            creator.clone(),
        )
        .unwrap();

        token_id_1 = dnr.token_id_1;
    }
    let sale_price = 1000_u128;
    let swap_msg = get_sim_swap_nfts_for_tokens_msg(
        collection,
        token_id_1,
        sale_price,
        true,
        user2.clone(),
        None,
    );
    let query_msg = QueryMsg::PoolQuotesSell {
        collection: collection.to_string(),
        query_options: QueryOptions {
            /// Whether to sort items in ascending or descending order
            descending: None,
            /// The key to start the query after
            start_after: None,
            // The number of items that will be returned
            limit: Some(5),
        },
    };
    let res: StdResult<PoolQuoteResponse> =
        router.wrap().query_wasm_smart(infinity_pool, &query_msg);
    let expected_pool_quotes = PoolQuoteResponse {
        pool_quotes: vec![
            PoolQuote {
                id: 2,
                collection: collection.clone(),
                quote_price: spot_price_2.into(),
            },
            PoolQuote {
                id: 1,
                collection: collection.clone(),
                quote_price: spot_price_1.into(),
            },
        ],
    };
    assert_eq!(res.unwrap(), expected_pool_quotes);

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    let highest_seller_price = spot_price_2 + Uint128::from(100u64).u128();
    let expected_royalty_price = 130_u128;
    let expected_network_fee = Uint128::from(highest_seller_price)
        .checked_multiply_ratio(2_u128, 100_u128)
        .unwrap()
        .u128();

    check_nft_sale(
        highest_seller_price,
        expected_royalty_price,
        expected_network_fee,
        0,
        swaps,
        creator,
        user2,
        token_id_1.to_string(),
    )
}
