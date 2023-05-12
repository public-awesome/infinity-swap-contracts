use crate::helpers::nft_functions::{mint_and_approve_many, validate_nft_owner};
use crate::helpers::pool_functions::prepare_pool_variations;
use crate::helpers::utils::{assert_error, assert_event};
use crate::setup::setup_infinity_contracts::{setup_infinity_test, InfinityTestSetup};

use cosmwasm_std::{Addr, Uint128};
use cw721::Cw721ExecuteMsg;
use cw_multi_test::Executor;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, NftDepositsResponse, QueryMsg as InfinityPoolQueryMsg,
};
use infinity_pool::state::PoolType;
use infinity_pool::ContractError;
use infinity_shared::InfinityError;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_withdraw_nfts_token_pool() {
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
        &marketplace.to_string(),
        &infinity_index.to_string(),
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

        let mut owner_token_ids = mint_and_approve_many(
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

        // Perform an invalid NFT deposit by sending directly to contract
        for owner_token_id in &owner_token_ids {
            let response = router.execute_contract(
                accts.owner.clone(),
                collection.clone(),
                &Cw721ExecuteMsg::TransferNft {
                    token_id: owner_token_id.to_string(),
                    recipient: infinity_pool.to_string(),
                },
                &[],
            );
            assert!(response.is_ok());
        }

        // NFT deposits are not registered on token pools
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
        assert!(nft_deposits_after.len() == 0);

        // Only owner can withdraw NFTs
        let response = router.execute_contract(
            accts.bidder.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawNfts {
                token_ids: owner_token_ids.clone(),
                asset_recipient: None,
            },
            &[],
        );
        assert_error(
            response,
            ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
        );

        // Owner can only transfer NFTs they hold in the pool
        let mut withdraw_token_ids = owner_token_ids.clone();
        withdraw_token_ids.extend(bidder_token_ids.clone());
        let response: Result<cw_multi_test::AppResponse, anyhow::Error> = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawNfts {
                token_ids: withdraw_token_ids,
                asset_recipient: None,
            },
            &[],
        );
        assert_error(response, InfinityError::NotNftOwner(infinity_pool.to_string()));

        // Owner can withdraw their NFTs
        let recipient = Addr::unchecked("recipient");

        // Pool owns NFTs
        for owner_token_id in &owner_token_ids {
            validate_nft_owner(&router, &collection, owner_token_id.to_string(), &infinity_pool)
        }

        // Split owner token ids into two vectors and withdraw half
        let other_owner_token_ids = owner_token_ids.split_off(owner_token_ids.len() / 2);
        let response: Result<cw_multi_test::AppResponse, anyhow::Error> = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawNfts {
                token_ids: owner_token_ids.clone(),
                asset_recipient: Some(recipient.to_string()),
            },
            &[],
        );
        assert!(response.is_ok());
        assert_event(response, "wasm-withdraw-nfts");

        // NFTs are transferred to the owner
        for owner_token_id in &owner_token_ids {
            validate_nft_owner(&router, &collection, owner_token_id.to_string(), &recipient)
        }

        // Owner can execute WithdrawAllNfts
        let response: Result<cw_multi_test::AppResponse, anyhow::Error> = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawAllNfts {
                asset_recipient: Some(recipient.to_string()),
            },
            &[],
        );
        assert!(response.is_ok());
        assert_event(response, "wasm-withdraw-nfts");

        // NFTs are transferred to the owner
        for owner_token_id in &other_owner_token_ids {
            validate_nft_owner(&router, &collection, owner_token_id.to_string(), &recipient)
        }
    }
}

#[test]
fn try_withdraw_nfts_nft_or_trade_pool() {
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
        &marketplace.to_string(),
        &infinity_index.to_string(),
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

        let mut owner_token_ids = mint_and_approve_many(
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

        // Only owner can withdraw NFTs
        let response = router.execute_contract(
            accts.bidder.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawNfts {
                token_ids: owner_token_ids.clone(),
                asset_recipient: None,
            },
            &[],
        );
        assert_error(
            response,
            ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
        );

        // Owner can only transfer NFTs they hold in the pool
        let mut withdraw_token_ids = owner_token_ids.clone();
        withdraw_token_ids.extend(bidder_token_ids.clone());
        let response: Result<cw_multi_test::AppResponse, anyhow::Error> = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawNfts {
                token_ids: withdraw_token_ids,
                asset_recipient: None,
            },
            &[],
        );
        assert_error(response, InfinityError::NotNftOwner(infinity_pool.to_string()));

        // Owner can withdraw their NFTs
        let recipient = Addr::unchecked("recipient");

        // Pool owns NFTs
        for owner_token_id in &owner_token_ids {
            validate_nft_owner(&router, &collection, owner_token_id.to_string(), &infinity_pool)
        }

        // Split owner token ids into two vectors and withdraw half
        let other_owner_token_ids = owner_token_ids.split_off(owner_token_ids.len() / 2);
        let response: Result<cw_multi_test::AppResponse, anyhow::Error> = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawNfts {
                token_ids: owner_token_ids.clone(),
                asset_recipient: Some(recipient.to_string()),
            },
            &[],
        );
        assert!(response.is_ok());
        assert_event(response, "wasm-withdraw-nfts");

        // NFTs are transferred to the owner
        for owner_token_id in &owner_token_ids {
            validate_nft_owner(&router, &collection, owner_token_id.to_string(), &recipient)
        }

        // NFT deposits are removed from the pool
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
        assert!(nft_deposits_after.len() == num_nfts as usize - owner_token_ids.len());

        // Owner can execute WithdrawAllNfts
        let response: Result<cw_multi_test::AppResponse, anyhow::Error> = router.execute_contract(
            accts.owner.clone(),
            infinity_pool.clone(),
            &InfinityPoolExecuteMsg::WithdrawAllNfts {
                asset_recipient: Some(recipient.to_string()),
            },
            &[],
        );
        assert!(response.is_ok());
        assert_event(response, "wasm-withdraw-nfts");

        // NFTs are transferred to the owner
        for owner_token_id in &other_owner_token_ids {
            validate_nft_owner(&router, &collection, owner_token_id.to_string(), &recipient)
        }

        // All NFT deposits are removed from the pool
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
        assert!(nft_deposits_after.len() == 0);
    }
}
