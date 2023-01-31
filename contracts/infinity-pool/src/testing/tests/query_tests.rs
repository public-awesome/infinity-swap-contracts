use std::vec;

use crate::msg::{
    ConfigResponse, PoolQuoteResponse, PoolsByIdResponse, PoolsResponse, QueryMsg, QueryOptions,
};
use crate::state::{Config, PoolQuote};
use crate::testing::helpers::fixtures::{create_and_activate_pool_fixtures, create_pool_fixtures};
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::{Addr, Uint128};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const USER: &str = "user";
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
    let user = Addr::unchecked(ASSET_ACCOUNT);
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
    let user = Addr::unchecked(ASSET_ACCOUNT);
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
    let user = Addr::unchecked(USER);
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
    let (mut router, creator, bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
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
        bidder,
        asset_account,
    );

    let pool_query = QueryMsg::PoolQuotesBuy {
        collection: collection.to_string(),
        query_options: QueryOptions {
            descending: Some(false),
            start_after: Some((Uint128::from(550u64), 5u64)),
            limit: Some(3),
        },
    };

    let res: PoolQuoteResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &pool_query)
        .unwrap();

    assert_eq!(res.pool_quotes.len(), 3);
    assert_eq!(
        res.pool_quotes[0],
        PoolQuote {
            id: 8,
            collection: collection.clone(),
            quote_price: Uint128::from(800u64)
        }
    );
    assert_eq!(
        res.pool_quotes[1],
        PoolQuote {
            id: 9,
            collection: collection.clone(),
            quote_price: Uint128::from(900u64)
        }
    );
    assert_eq!(
        res.pool_quotes[2],
        PoolQuote {
            id: 12,
            collection,
            quote_price: Uint128::from(1320u64)
        }
    );
}

#[test]
fn try_query_pools_by_sell_price() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let minter = vt.collection_response_vec[0].minter.clone().unwrap();
    // let user = Addr::unchecked(USER);
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
        bidder,
        asset_account,
    );

    let pool_query = QueryMsg::PoolQuotesSell {
        collection: collection.to_string(),
        query_options: QueryOptions {
            descending: Some(true),
            start_after: Some((Uint128::from(550u64), 5u64)),
            limit: Some(3),
        },
    };

    let res: PoolQuoteResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &pool_query)
        .unwrap();
    assert_eq!(res.pool_quotes.len(), 3);
    assert_eq!(
        res.pool_quotes[0],
        PoolQuote {
            id: 13,
            collection: collection.clone(),
            quote_price: Uint128::from(1300u64)
        }
    );
    assert_eq!(
        res.pool_quotes[1],
        PoolQuote {
            id: 12,
            collection: collection.clone(),
            quote_price: Uint128::from(1200u64)
        }
    );
    assert_eq!(
        res.pool_quotes[2],
        PoolQuote {
            id: 11,
            collection,
            quote_price: Uint128::from(1100u64)
        }
    );
}
