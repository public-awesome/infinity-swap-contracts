use crate::helpers::pool_functions::prepare_pool_variations;
use crate::helpers::utils::{assert_error, assert_event};
use crate::setup::setup_infinity_contracts::{setup_infinity_test, InfinityTestSetup};

use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, PoolConfigResponse, QueryMsg as InfinityPoolQueryMsg,
};
use infinity_pool::ContractError;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_set_is_active_token_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        infinity_global,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(1000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let pools_map = prepare_pool_variations(
        &mut router,
        infinity_pool_code_id,
        &accts.owner.to_string(),
        &collection.to_string(),
        &infinity_global.to_string(),
        &None,
        0u64,
        0u64,
        14,
        deposit_amount,
        vec![],
        0,
        false,
    );

    let pools = pools_map.iter().collect::<Vec<_>>();

    for (infinity_pool, _pool_config) in pools {
        // Non owner cannot activate pool
        let response = router.execute_contract(
            accts.bidder.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::SetIsActive {
                is_active: true,
            },
            &[],
        );
        assert_error(
            response,
            ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
        );

        // Owner can activate pool
        let response = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::SetIsActive {
                is_active: true,
            },
            &[],
        );
        assert_event(response, "wasm-set-is-active");

        let pool_config = router
            .wrap()
            .query_wasm_smart::<PoolConfigResponse>(
                infinity_pool.clone(),
                &&InfinityPoolQueryMsg::PoolConfig {},
            )
            .unwrap()
            .config;

        assert_eq!(pool_config.is_active, true);

        // Owner can deactive the pool
        let response = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::SetIsActive {
                is_active: false,
            },
            &[],
        );
        assert_event(response, "wasm-set-is-active");

        let pool_config = router
            .wrap()
            .query_wasm_smart::<PoolConfigResponse>(
                infinity_pool.clone(),
                &&InfinityPoolQueryMsg::PoolConfig {},
            )
            .unwrap()
            .config;

        assert_eq!(pool_config.is_active, false);
    }
}
