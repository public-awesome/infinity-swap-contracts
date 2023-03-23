use crate::helpers::{
    chain::Chain,
    constants::{INFINITY_SWAP_NAME, LISTING_FEE, MINT_PRICE, SG721_NAME},
    fixtures::get_pool_fixtures,
    helper::gen_users,
    instantiate::instantiate_minter,
    nft::{approve_all_nfts, mint_nfts},
    pool::{create_pools_from_fixtures, pool_execute_message, pool_query_message},
};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosmwasm_std::{Addr, Decimal, Uint128};
use infinity_swap::msg::{
    ExecuteMsg as InfinitySwapExecuteMsg, PoolsByIdResponse, QueryMsg as InfinitySwapQueryMsg,
};
use infinity_swap::state::{BondingCurve, Pool, PoolType};
use test_context::test_context;

#[test_context(Chain)]
#[test]
#[ignore]
fn test_small_pool_creation(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let prefix = chain.cfg.orc_cfg.chain_cfg.prefix.clone();

    // create minter with master account
    let master_account = chain.cfg.users[1].clone();

    // gen user that will:
    // * init minter
    // * mint 2 NFTs
    // * create Trade pool
    // * deposit 2 NFTs
    // * deposit 1_000_000 tokens
    // * activate pool
    // Min balance: MINT_PRICE + LISTING_FEE + 1_000_000 tokens
    let pool_deposit_amount = 1_000_000;
    let balance = (MINT_PRICE + LISTING_FEE + pool_deposit_amount) * 2;
    let user = gen_users(chain, 1, balance)[0].clone();
    let user_addr = user.to_addr(&prefix).unwrap();

    // init minter
    instantiate_minter(
        &mut chain.orc,
        // set creator address as user to allow for minting on base minter
        user_addr.to_string(),
        &master_account.key,
        &denom,
    )
    .unwrap();

    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    let token_ids = mint_nfts(chain, 2, &user);

    approve_all_nfts(
        chain,
        chain.orc.contract_map.address(INFINITY_SWAP_NAME).unwrap(),
        &user,
    );

    let resp = pool_execute_message(
        chain,
        InfinitySwapExecuteMsg::CreateTradePool {
            collection: collection.clone(),
            asset_recipient: None,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::zero(),
            delta: Uint128::zero(),
            finders_fee_bps: 0,
            swap_fee_bps: 100,
            reinvest_tokens: true,
            reinvest_nfts: true,
        },
        "infinity-swap-create-pool",
        vec![OrcCoin {
            amount: LISTING_FEE,
            denom: denom.parse().unwrap(),
        }],
        &user,
    );

    let tag = resp
        .res
        .find_event_tags("wasm-create-pool".to_string(), "id".to_string())[0];

    let pool_id = tag.value.parse::<u64>().unwrap();

    pool_execute_message(
        chain,
        InfinitySwapExecuteMsg::DepositNfts {
            pool_id,
            collection: collection.clone(),
            nft_token_ids: token_ids.clone(),
        },
        "infinity-swap-deposit-nfts",
        vec![],
        &user,
    );

    pool_execute_message(
        chain,
        InfinitySwapExecuteMsg::DepositTokens { pool_id },
        "infinity-swap-deposit-tokens",
        vec![OrcCoin {
            amount: pool_deposit_amount,
            denom: denom.parse().unwrap(),
        }],
        &user,
    );

    pool_execute_message(
        chain,
        InfinitySwapExecuteMsg::SetActivePool {
            is_active: true,
            pool_id,
        },
        "infinity-swap-activate",
        vec![],
        &user,
    );

    let resp: PoolsByIdResponse = pool_query_message(
        chain,
        InfinitySwapQueryMsg::PoolsById {
            pool_ids: vec![pool_id],
        },
    );
    let resp_pool = resp.pools[0].1.clone().unwrap();
    assert_eq!(
        resp_pool,
        Pool {
            id: pool_id,
            collection: Addr::unchecked(collection),
            owner: Addr::unchecked(user_addr.to_string()),
            asset_recipient: None,
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(500_000u128),
            delta: Uint128::zero(),
            total_tokens: Uint128::new(1000000u128),
            total_nfts: token_ids.len() as u64,
            finders_fee_percent: Decimal::zero(),
            swap_fee_percent: Decimal::new(Uint128::from(1000000000000000000u128)),
            is_active: true,
            reinvest_tokens: true,
            reinvest_nfts: true,
        }
    );
}

#[test_context(Chain)]
#[test]
#[ignore]
fn test_pool_creation_all_types(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let prefix = chain.cfg.orc_cfg.chain_cfg.prefix.clone();

    // create minter with master account
    let master_account = chain.cfg.users[1].clone();

    let pool_deposit_amount = 1_000_000;
    let balance = 1_000_000_000;
    let user = gen_users(chain, 1, balance)[0].clone();
    let user_addr = user.to_addr(&prefix).unwrap();

    let asset_account = gen_users(chain, 1, 1)[0].clone();
    let asset_account_addr = asset_account.to_addr(&prefix).unwrap();

    // init minter
    instantiate_minter(
        &mut chain.orc,
        // set creator address as user to allow for minting on base minter
        user_addr.to_string(),
        &master_account.key,
        &denom,
    )
    .unwrap();
    let collection = chain.orc.contract_map.address(SG721_NAME).unwrap();

    let pools = create_pools_from_fixtures(
        chain,
        &user,
        pool_deposit_amount,
        10,
        &Some(asset_account_addr.to_string()),
        150,
        300,
    );
    let fixtures = get_pool_fixtures(&collection, &Some(asset_account_addr.to_string()), 250, 300);

    assert_eq!(pools.len(), fixtures.len());
}
