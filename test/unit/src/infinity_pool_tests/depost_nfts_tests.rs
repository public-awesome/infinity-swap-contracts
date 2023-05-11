use crate::helpers::nft_functions::{mint_and_approve_many, validate_nft_owner};
use crate::helpers::pool_functions::create_pool;
use crate::helpers::utils::{assert_error, assert_event};
use crate::setup::setup_infinity_contracts::{setup_infinity_test, InfinityTestSetup};

use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, NftDepositsResponse, PoolConfigResponse, PoolInfo,
    QueryMsg as InfinityPoolQueryMsg,
};
use infinity_pool::state::{BondingCurve, PoolType};
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
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

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

#[test]
fn try_deposit_nfts_trade_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

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
        accts.bidder,
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

#[test]
fn try_deposit_nfts_nft_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

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
        accts.bidder,
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
