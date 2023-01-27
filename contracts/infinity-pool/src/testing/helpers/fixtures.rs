use crate::state::{BondingCurve, Pool, PoolType};
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::create_pool;
use cosmwasm_std::{Addr, MessageInfo, Uint128};
use sg_multi_test::StargazeApp;
use std::collections::{BTreeMap, BTreeSet};

use super::pool_functions::{activate, deposit_nfts, deposit_tokens};

pub fn get_pool_fixtures(
    collection: Addr,
    creator: Addr,
    user: Addr,
    asset_account: Addr,
) -> Vec<Pool> {
    vec![
        Pool {
            id: 1,
            collection: collection.clone(),
            owner: creator.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(100u64),
            delta: Uint128::from(10u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 2,
            collection: collection.clone(),
            owner: creator.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(200u64),
            delta: Uint128::from(20u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 3,
            collection: collection.clone(),
            owner: creator.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(300u64),
            delta: Uint128::from(30u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 4,
            collection: collection.clone(),
            owner: creator.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(400u64),
            delta: Uint128::from(40u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 5,
            collection: collection.clone(),
            owner: creator.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(500u64),
            delta: Uint128::from(50u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: Some(50u16),
            is_active: false,
        },
        Pool {
            id: 6,
            collection: collection.clone(),
            owner: creator.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(600u64),
            delta: Uint128::from(60u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: Some(60u16),
            is_active: false,
        },
        Pool {
            id: 7,
            collection: collection.clone(),
            owner: creator.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(700u64),
            delta: Uint128::from(70u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: Some(70u16),
            is_active: false,
        },
        Pool {
            id: 8,
            collection: collection.clone(),
            owner: user.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(800u64),
            delta: Uint128::from(80u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 9,
            collection: collection.clone(),
            owner: user.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(900u64),
            delta: Uint128::from(90u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 10,
            collection: collection.clone(),
            owner: user.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(1000u64),
            delta: Uint128::from(100u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 11,
            collection: collection.clone(),
            owner: user.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(1100u64),
            delta: Uint128::from(110u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: None,
            is_active: false,
        },
        Pool {
            id: 12,
            collection: collection.clone(),
            owner: user.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(1200u64),
            delta: Uint128::from(120u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: Some(50u16),
            is_active: false,
        },
        Pool {
            id: 13,
            collection: collection.clone(),
            owner: user.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(1300u64),
            delta: Uint128::from(130u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: Some(60u16),
            is_active: false,
        },
        Pool {
            id: 14,
            collection: collection.clone(),
            owner: user.clone(),
            asset_recipient: Some(asset_account.clone()),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(1400u64),
            delta: Uint128::from(140u64),
            total_tokens: Uint128::from(0u64),
            nft_token_ids: BTreeSet::new(),
            fee_bps: Some(70u16),
            is_active: false,
        },
    ]
}

pub fn create_pool_fixtures(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    collection: Addr,
    creator: Addr,
    user: Addr,
    asset_account: Addr,
) -> Vec<Pool> {
    let pools = get_pool_fixtures(collection, creator, user, asset_account);
    for pool in pools.iter() {
        create_pool(
            router,
            infinity_pool.clone(),
            pool.owner.clone(),
            pool.collection.clone(),
            pool.asset_recipient.clone(),
            pool.pool_type.clone(),
            pool.bonding_curve.clone(),
            pool.spot_price,
            pool.delta,
            pool.fee_bps,
        )
        .unwrap();
    }
    pools
}

pub fn create_and_activate_pool_fixtures(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    minter: Addr,
    collection: Addr,
    creator: Addr,
    user: Addr,
    asset_account: Addr,
) -> Vec<Pool> {
    let pools = get_pool_fixtures(collection.clone(), creator.clone(), user, asset_account);
    for pool in pools.iter() {
        create_pool(
            router,
            infinity_pool.clone(),
            pool.owner.clone(),
            pool.collection.clone(),
            pool.asset_recipient.clone(),
            pool.pool_type.clone(),
            pool.bonding_curve.clone(),
            pool.spot_price,
            pool.delta,
            pool.fee_bps,
        )
        .unwrap();
        if pool.can_buy_nfts() {
            deposit_tokens(
                router,
                infinity_pool.clone(),
                pool.owner.clone(),
                pool.id,
                pool.spot_price * Uint128::from(2u64),
            )
            .unwrap();
        }
        if pool.can_sell_nfts() {
            let token_id_1 = mint(router, &pool.owner, &minter);
            approve(
                router,
                &pool.owner,
                &collection,
                &infinity_pool,
                token_id_1.clone(),
            );
            let token_id_2 = mint(router, &pool.owner, &minter);
            approve(
                router,
                &pool.owner,
                &collection,
                &infinity_pool,
                token_id_2.clone(),
            );
            deposit_nfts(
                router,
                infinity_pool.clone(),
                pool.owner.clone(),
                pool.id,
                pool.collection.clone(),
                vec![token_id_1, token_id_2]
                    .iter()
                    .map(|t| t.to_string())
                    .collect(),
            )
            .unwrap();
        }
        activate(
            router,
            infinity_pool.clone(),
            pool.owner.clone(),
            pool.id,
            true,
        )
        .unwrap();
    }
    pools
}
