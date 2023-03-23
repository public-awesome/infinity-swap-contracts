use crate::helpers::{
    chain::Chain,
    constants::{INFINITY_SWAP_NAME, LISTING_FEE, SG721_NAME},
    helper::{gen_users, latest_block_time},
    instantiate::instantiate_minter,
    nft::{approve_all_nfts, mint_nfts},
    pool::{create_active_pool, pool_execute_message, pool_query_message},
};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosmwasm_std::{Addr, Decimal, Uint128};
use infinity_swap::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, PoolsByIdResponse, QueryMsg as InfinityPoolQueryMsg,
    SwapParams,
};
use infinity_swap::state::Pool;
use infinity_swap::state::{BondingCurve, PoolType};
use std::env;
use test_context::test_context;

#[allow(dead_code)]
const LARGE_NUM_SWAPS: usize = 250;

#[test_context(Chain)]
#[test]
#[ignore]
fn large_single_pool_token_for_nft_swap(chain: &mut Chain) {
    if env::var("ENABLE_LARGE_TESTS").is_err() {
        return;
    }

    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let prefix = chain.cfg.orc_cfg.chain_cfg.prefix.clone();

    let master_account = chain.cfg.users[1].clone();

    let pool_deposit_amount = 10_000_000;
    let balance = pool_deposit_amount * 10_000;
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

    let token_ids = mint_nfts(chain, 10_000, &user);

    approve_all_nfts(
        chain,
        chain.orc.contract_map.address(INFINITY_SWAP_NAME).unwrap(),
        &user,
    );

    let resp = pool_execute_message(
        chain,
        InfinityPoolExecuteMsg::CreateTradePool {
            collection: collection.clone(),
            asset_recipient: None,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::zero(),
            delta: Uint128::zero(),
            finders_fee_bps: 0,
            swap_fee_bps: 0,
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
        InfinityPoolExecuteMsg::DepositNfts {
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
        InfinityPoolExecuteMsg::DepositTokens { pool_id },
        "infinity-swap-deposit-tokens",
        vec![OrcCoin {
            amount: pool_deposit_amount,
            denom: denom.parse().unwrap(),
        }],
        &user,
    );

    pool_execute_message(
        chain,
        InfinityPoolExecuteMsg::SetActivePool {
            is_active: true,
            pool_id,
        },
        "infinity-swap-activate",
        vec![],
        &user,
    );

    let resp: PoolsByIdResponse = pool_query_message(
        chain,
        InfinityPoolQueryMsg::PoolsById {
            pool_ids: vec![pool_id],
        },
    );
    let resp_pool = resp.pools[0].1.clone().unwrap();

    assert_eq!(
        resp_pool,
        Pool {
            id: 1,
            collection: Addr::unchecked(collection.clone()),
            owner: Addr::unchecked(user_addr.to_string()),
            asset_recipient: None,
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::new(pool_deposit_amount) / Uint128::from(token_ids.len() as u64),
            delta: Uint128::zero(),
            total_tokens: Uint128::new(pool_deposit_amount),
            total_nfts: token_ids.len() as u64,
            finders_fee_percent: Decimal::zero(),
            swap_fee_percent: Decimal::zero(),
            is_active: true,
            reinvest_tokens: true,
            reinvest_nfts: true,
        }
    );

    let max_expected_token_input = [Uint128::from(10_000u64); LARGE_NUM_SWAPS];
    let send_amount: Uint128 = max_expected_token_input.iter().sum();
    let exec_res = pool_execute_message(
        chain,
        InfinityPoolExecuteMsg::SwapTokensForAnyNfts {
            collection,
            max_expected_token_input: max_expected_token_input.to_vec(),
            swap_params: SwapParams {
                deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                robust: false,
                asset_recipient: None,
                finder: None,
            },
        },
        "infinity-swap-swap-tokens-for-any-nfts",
        vec![OrcCoin {
            amount: send_amount.u128(),
            denom: denom.parse().unwrap(),
        }],
        &user,
    );
    println!("gas_wanted {:?}", exec_res.res.gas_wanted);
    println!("gas_used {:?}", exec_res.res.gas_used);
}

#[test_context(Chain)]
#[test]
#[ignore]
fn large_many_pool_token_for_nft_swap(chain: &mut Chain) {
    if env::var("ENABLE_LARGE_TESTS").is_err() {
        return;
    }

    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let prefix = chain.cfg.orc_cfg.chain_cfg.prefix.clone();

    let master_account = chain.cfg.users[1].clone();

    let pool_deposit_amount = 10_000_000;
    let balance = pool_deposit_amount * 10_000;
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

    approve_all_nfts(
        chain,
        chain.orc.contract_map.address(INFINITY_SWAP_NAME).unwrap(),
        &user,
    );

    let mut pools: Vec<Pool> = vec![];
    for _ in 0..LARGE_NUM_SWAPS {
        pools.push(create_active_pool(
            chain,
            &user,
            pool_deposit_amount,
            2,
            InfinityPoolExecuteMsg::CreateNftPool {
                collection: collection.to_string(),
                asset_recipient: None,
                bonding_curve: BondingCurve::Linear,
                spot_price: Uint128::from(100u64),
                delta: Uint128::from(10u64),
                finders_fee_bps: 0,
            },
        ));
    }

    let max_expected_token_input = [Uint128::from(10_000u64); LARGE_NUM_SWAPS];
    let send_amount: Uint128 = max_expected_token_input.iter().sum();
    let exec_res = pool_execute_message(
        chain,
        InfinityPoolExecuteMsg::SwapTokensForAnyNfts {
            collection,
            max_expected_token_input: max_expected_token_input.to_vec(),
            swap_params: SwapParams {
                deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                robust: false,
                asset_recipient: None,
                finder: None,
            },
        },
        "infinity-swap-swap-tokens-for-any-nfts",
        vec![OrcCoin {
            amount: send_amount.u128(),
            denom: denom.parse().unwrap(),
        }],
        &user,
    );
    println!("gas_wanted {:?}", exec_res.res.gas_wanted);
    println!("gas_used {:?}", exec_res.res.gas_used);
}
