use std::vec;

use crate::msg::{
    ConfigResponse, NftTokenIdsResponse, PoolQuoteResponse, PoolsByIdResponse, PoolsResponse,
    QueryMsg, QueryOptions,
};
use crate::state::{BondingCurve, Config, PoolQuote};
use crate::testing::helpers::fixtures::{create_and_activate_pool_fixtures, create_pool_fixtures};
use crate::testing::helpers::nft_functions::mint_and_approve_many;
use crate::testing::helpers::pool_functions::prepare_swap_pool;
use crate::testing::setup::setup_accounts::setup_addtl_account;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::{Addr, Uint128};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const USER: &str = "user1";
const ASSET_ACCOUNT: &str = "asset";

#[test]
fn try_query_config() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let _collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let _asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator, marketplace.clone()).unwrap();

    let config_query = QueryMsg::Config {};

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &config_query)
        .unwrap();

    assert_eq!(
        res.config,
        Config {
            denom: NATIVE_DENOM.to_string(),
            marketplace_addr: marketplace,
            developer: None,
        }
    )
}

#[test]
fn try_query_pools() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let user = setup_addtl_account(&mut router, USER, 1_000_000).unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pools = create_pool_fixtures(
        &mut router,
        infinity_pool.clone(),
        collection,
        creator,
        user,
        asset_account,
        0,
        50,
    );

    let pool_query = QueryMsg::Pools {
        query_options: QueryOptions {
            descending: Some(false),
            start_after: Some(3u64),
            limit: Some(2),
        },
    };

    let res: PoolsResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &pool_query)
        .unwrap();
    assert_eq!(res.pools.len(), 2);
    assert_eq!(res.pools[0], pools[3].clone());
    assert_eq!(res.pools[1], pools[4].clone());
}

#[test]
fn try_query_pools_by_id() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let user = setup_addtl_account(&mut router, USER, 1_000_000).unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pools = create_pool_fixtures(
        &mut router,
        infinity_pool.clone(),
        collection,
        creator,
        user,
        asset_account,
        0,
        50,
    );

    let pool_query = QueryMsg::PoolsById {
        pool_ids: vec![1, 2, 1000],
    };

    let res: PoolsByIdResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &pool_query)
        .unwrap();
    assert_eq!(res.pools[0], (1, Some(pools[0].clone())));
    assert_eq!(res.pools[1], (2, Some(pools[1].clone())));
    assert_eq!(res.pools[2], (1000, None));
}

#[test]
fn try_query_pools_by_owner() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let user = setup_addtl_account(&mut router, USER, 1_000_000).unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pools = create_pool_fixtures(
        &mut router,
        infinity_pool.clone(),
        collection,
        creator,
        user.clone(),
        asset_account,
        0,
        50,
    );

    let pool_query = QueryMsg::PoolsByOwner {
        owner: user.to_string(),
        query_options: QueryOptions {
            descending: Some(false),
            start_after: Some(9u64),
            limit: Some(3),
        },
    };

    let res: PoolsResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &pool_query)
        .unwrap();
    assert_eq!(res.pools.len(), 3);
    assert_eq!(res.pools[0], pools[9].clone());
    assert_eq!(res.pools[1], pools[10].clone());
    assert_eq!(res.pools[2], pools[11].clone());
}

#[test]
fn try_query_pools_by_buy_price() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let minter = vt.collection_response_vec[0].minter.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    let _pools = create_and_activate_pool_fixtures(
        &mut router,
        infinity_pool.clone(),
        minter,
        collection.clone(),
        creator,
        asset_account,
        0,
        50,
    );

    let pool_query = QueryMsg::PoolQuotesBuy {
        collection: collection.to_string(),
        query_options: QueryOptions {
            descending: None,
            start_after: None,
            limit: None,
        },
    };

    let res: PoolQuoteResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &pool_query)
        .unwrap();

    assert_eq!(res.pool_quotes.len(), 10);
    let expected_pool_quotes = vec![
        PoolQuote {
            id: 13,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1469u128),
        },
        PoolQuote {
            id: 12,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1320u128),
        },
        PoolQuote {
            id: 14,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1000u128),
        },
        PoolQuote {
            id: 7,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1000u128),
        },
        PoolQuote {
            id: 9,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(900u128),
        },
        PoolQuote {
            id: 8,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(800u128),
        },
        PoolQuote {
            id: 6,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(636u128),
        },
        PoolQuote {
            id: 5,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(550u128),
        },
        PoolQuote {
            id: 2,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(200u128),
        },
        PoolQuote {
            id: 1,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(100u128),
        },
    ];

    for (id, pq) in res.pool_quotes.iter().enumerate() {
        assert_eq!(pq, &expected_pool_quotes[id]);
    }
}

#[test]
fn try_query_pools_by_sell_price() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let minter = vt.collection_response_vec[0].minter.clone().unwrap();
    let _user = setup_addtl_account(&mut router, USER, 1_000_000).unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    let _pools = create_and_activate_pool_fixtures(
        &mut router,
        infinity_pool.clone(),
        minter,
        collection.clone(),
        creator,
        asset_account,
        0,
        50,
    );

    let pool_query = QueryMsg::PoolQuotesSell {
        collection: collection.to_string(),
        query_options: QueryOptions {
            descending: None,
            start_after: None,
            limit: None,
        },
    };

    let res: PoolQuoteResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &pool_query)
        .unwrap();
    assert_eq!(res.pool_quotes.len(), 10);
    let expected_pool_quotes = vec![
        PoolQuote {
            id: 3,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(300u64),
        },
        PoolQuote {
            id: 4,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(400u64),
        },
        PoolQuote {
            id: 5,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(500u64),
        },
        PoolQuote {
            id: 6,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(600u64),
        },
        PoolQuote {
            id: 10,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1000u64),
        },
        PoolQuote {
            id: 11,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1100u64),
        },
        PoolQuote {
            id: 12,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1200u64),
        },
        PoolQuote {
            id: 13,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(1300u64),
        },
        PoolQuote {
            id: 7,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(3000u64),
        },
        PoolQuote {
            id: 14,
            collection: Addr::unchecked("contract2"),
            quote_price: Uint128::from(3000u64),
        },
    ];
    for (id, pq) in res.pool_quotes.iter().enumerate() {
        assert_eq!(pq, &expected_pool_quotes[id]);
    }
}

#[test]
fn try_query_pool_nft_token_ids() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let minter = vt.collection_response_vec[0].minter.clone().unwrap();
    let owner = setup_addtl_account(&mut router, USER, 1_000_000_000).unwrap();
    let _asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &creator,
        &owner,
        &minter,
        &collection,
        &infinity_pool,
        500,
    );

    let pool = prepare_swap_pool(
        &mut router,
        &infinity_pool.clone(),
        &owner,
        Uint128::from(1_000_000u128),
        owner_token_ids.clone(),
        true,
        crate::msg::ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: None,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::zero(),
            delta: Uint128::zero(),
            finders_fee_bps: 0,
            swap_fee_bps: 0,
            reinvest_tokens: true,
            reinvest_nfts: true,
        },
    )
    .unwrap();

    let mut expected_token_ids = vec![];
    let mut start_after = None;
    loop {
        let response: NftTokenIdsResponse = router
            .wrap()
            .query_wasm_smart(
                infinity_pool.clone(),
                &QueryMsg::PoolNftTokenIds {
                    pool_id: pool.id,
                    query_options: QueryOptions {
                        descending: None,
                        start_after: start_after.clone(),
                        limit: None,
                    },
                },
            )
            .unwrap();
        if response.nft_token_ids.is_empty() {
            break;
        }
        start_after = Some(response.nft_token_ids.last().unwrap().to_string());
        expected_token_ids.extend(response.nft_token_ids);
    }
    assert_eq!(expected_token_ids.len(), owner_token_ids.len());
}
