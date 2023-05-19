use crate::helpers::nft_functions::{mint_and_approve_many, validate_nft_owner};
use crate::helpers::pool_functions::prepare_pool_variations;
use crate::helpers::utils::{assert_error, assert_event};
use crate::setup::setup_infinity_contracts::{setup_infinity_test, InfinityTestSetup};

use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, NftDepositsResponse, PoolConfigResponse,
    QueryMsg as InfinityPoolQueryMsg,
};
use infinity_pool::state::PoolType;
use infinity_pool::ContractError;
use infinity_shared::InfinityError;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_deposit_nfts_token_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_global,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(1000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
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

    let pools =
        pools_map.iter().filter(|(_, &ref pc)| pc.pool_type == PoolType::Token).collect::<Vec<_>>();

    for (infinity_pool, _pool_config) in pools {
        let num_nfts = 10u32;

        let owner_token_ids = mint_and_approve_many(
            &mut router,
            &accts.creator,
            &accts.owner,
            &minter,
            &collection,
            &infinity_pool,
            num_nfts,
        );

        let response = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::DepositNfts {
                collection: collection.to_string(),
                token_ids: owner_token_ids.clone(),
            },
            &[],
        );

        assert_error(response, ContractError::InvalidPool("pool cannot escrow NFTs".to_string()));
    }
}

#[test]
fn try_deposit_nfts_nft_or_trade_pool() {
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
    let minter = collection_resp.minter.clone().unwrap();
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

    let pools = pools_map
        .iter()
        .filter(|(_, &ref pc)| pc.pool_type == PoolType::Nft || pc.pool_type == PoolType::Trade)
        .collect::<Vec<_>>();

    for (infinity_pool, _pool_config) in pools {
        let num_nfts = 10u32;

        let owner_token_ids = mint_and_approve_many(
            &mut router,
            &accts.creator,
            &accts.owner,
            &minter,
            &collection,
            &infinity_pool,
            num_nfts,
        );

        let bidder_token_ids = mint_and_approve_many(
            &mut router,
            &accts.creator,
            &accts.bidder,
            &minter,
            &collection,
            &infinity_pool,
            num_nfts,
        );

        // Deposting as non-owner of pool should return an error
        let response = router.execute_contract(
            accts.bidder.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::DepositNfts {
                collection: collection.to_string(),
                token_ids: vec![bidder_token_ids.first().unwrap().to_string()],
            },
            &[],
        );
        assert_error(
            response,
            ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
        );

        // Deposting as non-owner of nft should return an error
        let response = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::DepositNfts {
                collection: collection.to_string(),
                token_ids: vec![bidder_token_ids.first().unwrap().to_string()],
            },
            &[],
        );
        assert_error(response, InfinityError::NotNftOwner(accts.owner.to_string()));

        // Deposting an empty list returns an error
        let response = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::DepositNfts {
                collection: collection.to_string(),
                token_ids: vec![],
            },
            &[],
        );
        assert_error(
            response,
            ContractError::InvalidInput("token_ids should not be empty".to_string()),
        );

        // Valid deposit emits correct events, updates the pool, and transfers the nfts

        // Before depositing, the pool total nft count should be 0
        let pool_config_before = router
            .wrap()
            .query_wasm_smart::<PoolConfigResponse>(
                infinity_pool.clone(),
                &&InfinityPoolQueryMsg::PoolConfig {},
            )
            .unwrap()
            .config;
        assert_eq!(pool_config_before.total_nfts, 0u64);

        // Before depositing, the nft deposits map should be empty
        let nft_deposits_before = router
            .wrap()
            .query_wasm_smart::<NftDepositsResponse>(
                infinity_pool.clone(),
                &&InfinityPoolQueryMsg::NftDeposits {
                    query_options: None,
                },
            )
            .unwrap()
            .nft_deposits;
        assert!(nft_deposits_before.is_empty());

        // Execute deposit nfts
        let response = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::DepositNfts {
                collection: collection.to_string(),
                token_ids: owner_token_ids.clone(),
            },
            &[],
        );

        assert!(response.is_ok());
        assert_event(response, "wasm-deposit-nfts");

        // After depositing, the pool total nft count should be incremented
        let pool_config_after = router
            .wrap()
            .query_wasm_smart::<PoolConfigResponse>(
                infinity_pool.clone(),
                &&InfinityPoolQueryMsg::PoolConfig {},
            )
            .unwrap()
            .config;
        assert_eq!(pool_config_after.total_nfts, num_nfts as u64);

        // The infinity pool address should now be the owner of the NFTs
        for token_id in &owner_token_ids {
            validate_nft_owner(&router, &collection, token_id.to_string(), &infinity_pool);
        }

        // After depositing, the nft deposits map should be non-empty
        let nft_deposits_after = router
            .wrap()
            .query_wasm_smart::<NftDepositsResponse>(
                infinity_pool.clone(),
                &&InfinityPoolQueryMsg::NftDeposits {
                    query_options: None,
                },
            )
            .unwrap()
            .nft_deposits;
        assert!(nft_deposits_after.len() == num_nfts as usize);

        // Once deposited, nfts cannot be deposited again
        let response = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::DepositNfts {
                collection: collection.to_string(),
                token_ids: owner_token_ids.clone(),
            },
            &[],
        );
        assert_error(response, InfinityError::NotNftOwner(accts.owner.to_string()));
    }
}
