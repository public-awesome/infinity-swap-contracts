use crate::helpers::fixtures::get_pool_fixtures;

use cosmwasm_std::{Addr, BankMsg, Coin, Uint128};
use cw_multi_test::Executor;
use infinity_pool::{
    msg::{
        InstantiateMsg as InfinityPoolInstantiateMsg, PoolConfigResponse, PoolInfo,
        QueryMsg as InfinityPoolQueryMsg,
    },
    state::{PoolConfig, PoolType},
};
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;
use std::collections::HashMap;

pub fn create_pool(
    router: &mut StargazeApp,
    infinity_pool_code_id: u64,
    owner: &str,
    infinity_global: &str,
    pool_info: PoolInfo,
    deposit_tokens: Uint128,
    is_active: bool,
) -> (Addr, PoolConfig) {
    let response = router.instantiate_contract(
        infinity_pool_code_id,
        Addr::unchecked(owner.clone()),
        &InfinityPoolInstantiateMsg {
            infinity_global: infinity_global.to_string(),
            pool_info,
        },
        &[],
        "InfinityPool",
        None,
    );
    assert!(response.is_ok());
    let infinity_pool = response.unwrap();

    if deposit_tokens > Uint128::zero() {
        let response = router.execute(
            Addr::unchecked(owner.clone()),
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
                to_address: infinity_pool.to_string(),
                amount: vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: deposit_tokens,
                }],
            }),
        );
        assert!(response.is_ok());
    }

    if is_active {
        let response = router.execute_contract(
            Addr::unchecked(owner.clone()),
            infinity_pool.clone(),
            &infinity_pool::msg::ExecuteMsg::SetIsActive {
                is_active: true,
            },
            &[],
        );
        assert!(response.is_ok());
    }

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    (infinity_pool, pool_config)
}

pub fn prepare_pool_variations(
    router: &mut StargazeApp,
    infinity_pool_code_id: u64,
    owner: &str,
    collection: &str,
    infinity_global: &str,
    asset_recipient: &Option<String>,
    finders_fee_bps: u64,
    swap_fee_bps: u64,
    num_pools: u8,
    deposit_tokens_per_pool: Uint128,
    mut nft_token_ids: Vec<String>,
    nfts_per_pool: u8,
    is_active: bool,
) -> HashMap<Addr, PoolConfig> {
    let mut pools_map: HashMap<Addr, PoolConfig> = HashMap::new();

    let pool_fixtures =
        &get_pool_fixtures(owner, collection, asset_recipient, finders_fee_bps, swap_fee_bps)
            [0..(num_pools as usize)];

    for fixt in pool_fixtures {
        let (addr, pool_config) = create_pool(
            router,
            infinity_pool_code_id,
            owner,
            infinity_global,
            fixt.clone(),
            deposit_tokens_per_pool,
            is_active,
        );
        pools_map.insert(addr, pool_config);
    }
    pools_map
}

// pub fn deposit_tokens(
//     router: &mut StargazeApp,
//     infinity_swap: Addr,
//     owner: Addr,
//     pool_id: u64,
//     deposit_amount: Uint128,
// ) -> Result<u128, Error> {
//     let msg = InfinitySwapExecuteMsg::DepositTokens {
//         pool_id,
//     };
//     let res = router.execute_contract(
//         owner,
//         infinity_swap,
//         &msg,
//         &coins(deposit_amount.u128(), NATIVE_DENOM),
//     );
//     assert!(res.is_ok());
//     let total_tokens = res.unwrap().events[1].attributes[3].value.parse::<u128>().unwrap();

//     Ok(total_tokens)
// }

// pub fn deposit_nfts(
//     router: &mut StargazeApp,
//     infinity_swap: Addr,
//     owner: Addr,
//     pool_id: u64,
//     collection: Addr,
//     nft_token_ids: Vec<String>,
// ) -> Result<String, Error> {
//     let msg = InfinitySwapExecuteMsg::DepositNfts {
//         pool_id,
//         collection: collection.to_string(),
//         nft_token_ids,
//     };
//     let res = router.execute_contract(owner, infinity_swap, &msg, &[]);
//     assert!(res.is_ok());
//     let nft_token_ids = res.unwrap().events[1].attributes[2].value.clone();

//     Ok(nft_token_ids)
// }

// pub fn activate(
//     router: &mut StargazeApp,
//     infinity_swap: &Addr,
//     owner: &Addr,
//     pool_id: u64,
//     is_active: bool,
// ) -> Result<Pool, Error> {
//     let msg = InfinitySwapExecuteMsg::SetActivePool {
//         pool_id,
//         is_active,
//     };
//     let res = router.execute_contract(owner.clone(), infinity_swap.clone(), &msg, &[]);
//     assert!(res.is_ok());
//     let query_msg = QueryMsg::PoolsById {
//         pool_ids: vec![pool_id],
//     };
//     let res: PoolsByIdResponse = router.wrap().query_wasm_smart(infinity_swap, &query_msg).unwrap();

//     let pool = res.pools[0].1.clone().unwrap();
//     Ok(pool)
// }

// pub fn prepare_swap_pool(
//     router: &mut StargazeApp,
//     infinity_swap: &Addr,
//     owner: &Addr,
//     num_deposit_tokens: Uint128,
//     nft_token_ids: Vec<String>,
//     is_active: bool,
//     create_pool_msg: InfinitySwapExecuteMsg,
// ) -> Result<Pool, Error> {
//     let pool = create_pool(router, infinity_swap.clone(), owner.clone(), create_pool_msg)?;

//     if num_deposit_tokens > Uint128::zero() {
//         deposit_tokens(router, infinity_swap.clone(), owner.clone(), pool.id, num_deposit_tokens)?;
//     }

//     if !nft_token_ids.is_empty() {
//         deposit_nfts(
//             router,
//             infinity_swap.clone(),
//             owner.clone(),
//             pool.id,
//             pool.collection.clone(),
//             nft_token_ids,
//         )?;
//     }

//     let pool = activate(router, infinity_swap, owner, pool.id, is_active)?;

//     Ok(pool)
// }
