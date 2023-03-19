use crate::msg::QueryMsg::SimDirectSwapTokensForSpecificNfts;
use crate::msg::SwapResponse;
use crate::msg::{NftSwap, SwapParams};
use crate::testing::helpers::nft_functions::mint_and_approve_many;
use crate::testing::helpers::pool_functions::prepare_pool_variations;
use crate::testing::helpers::swap_functions::{setup_swap_test, validate_swap_fees, SwapTestSetup};
use crate::testing::setup::setup_accounts::setup_addtl_account;
use cosmwasm_std::{StdError, StdResult, Timestamp, Uint128};
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::GENESIS_MINT_START_TIME;
use test_suite::common_setup::msg::VendingTemplateResponse;

#[test]
fn cant_swap_inactive_pool() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        false,
        0,
        0,
    );

    for pool in pools {
        if !pool.can_sell_nfts() {
            continue;
        }
        let nfts_to_swap_for: Vec<NftSwap> = pool
            .nft_token_ids
            .into_iter()
            .take(3)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(100_000u128),
            })
            .collect();
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: true,
                asset_recipient: None,
                finder: None,
            },
        };

        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);
        assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: "Querier contract error: Generic error: Invalid pool: pool is not active"
                    .to_string()
            }
        );
    }
}

#[test]
fn cant_swap_invalid_pool_type() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        false,
        0,
        0,
    );

    for pool in pools {
        if pool.can_sell_nfts() {
            continue;
        }
        let nfts_to_swap_for: Vec<NftSwap> = pool
            .nft_token_ids
            .into_iter()
            .take(3)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(100_000u128),
            })
            .collect();
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: true,
                asset_recipient: None,
                finder: None,
            },
        };

        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);
        assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: "Querier contract error: Generic error: Invalid pool: pool cannot sell nfts"
                    .to_string()
            }
        );
    }
}

#[test]
fn can_swap_active_pool() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    for pool in pools {
        if !pool.can_sell_nfts() {
            continue;
        }
        let nfts_to_swap_for: Vec<NftSwap> = pool
            .nft_token_ids
            .into_iter()
            .take(3)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(100_000u128),
            })
            .collect();
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: false,
                asset_recipient: None,
                finder: None,
            },
        };

        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);
        assert_eq!(res.unwrap().swaps.len(), 3);
    }
}

#[test]
fn incorrect_nfts_error() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let mut bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    for pool in pools {
        if !pool.can_sell_nfts() {
            continue;
        }
        let nfts_to_swap_for: Vec<NftSwap> = bidder_token_ids
            .drain(0..3)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(100_000u128),
            })
            .collect();
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for: nfts_to_swap_for.clone(),
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: false,
                asset_recipient: None,
                finder: None,
            },
        };

        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);
        assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: format!(
                    "Querier contract error: Generic error: Swap error: pool does not own NFT {}",
                    nfts_to_swap_for[0].nft_token_id
                )
                .to_string()
            }
        );
    }
}

#[test]
fn sale_price_above_max_expected() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    for pool in pools {
        if !pool.can_sell_nfts() {
            continue;
        }
        let nfts_to_swap_for: Vec<NftSwap> = pool
            .nft_token_ids
            .into_iter()
            .take(3)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(20u128),
            })
            .collect();
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: false,
                asset_recipient: None,
                finder: None,
            },
        };

        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);

        assert_eq!(res.unwrap_err(), StdError::GenericErr {
            msg: "Querier contract error: Generic error: Swap error: pool sale price is above max expected"
                .to_string(),
        });
    }
}

#[test]
fn robust_query_does_not_revert_whole_tx() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    for pool in pools {
        if !pool.can_sell_nfts() {
            continue;
        }
        let mut pool_nft_token_ids: Vec<String> = pool.nft_token_ids.into_iter().collect();
        let mut nfts_to_swap_for: Vec<NftSwap> = pool_nft_token_ids
            .drain(0..2_usize)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(100_000_u128),
            })
            .collect();
        nfts_to_swap_for.extend(
            pool_nft_token_ids
                .drain(0..1_usize)
                .map(|token_id| NftSwap {
                    nft_token_id: token_id,
                    token_amount: Uint128::from(10_u128),
                })
                .collect::<Vec<NftSwap>>(),
        );
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: true,
                asset_recipient: None,
                finder: None,
            },
        };

        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);
        assert_eq!(res.unwrap().swaps.len(), 2);
    }
}

#[test]
fn minimal_fee_tx_is_handled_correctly() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        marketplace,
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    for pool in pools {
        if !pool.can_sell_nfts() {
            continue;
        }
        let nfts_to_swap_for: Vec<NftSwap> = pool
            .nft_token_ids
            .clone()
            .into_iter()
            .take(3)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(100_000_u128),
            })
            .collect();
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: true,
                asset_recipient: None,
                finder: None,
            },
        };
        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);

        for swap in res.unwrap().swaps {
            validate_swap_fees(
                &swap,
                &pool,
                &marketplace_params,
                &collection_info.royalty_info,
            );
        }
    }
}

#[test]
fn finders_and_swap_fee_tx_is_handled_correctly() {
    let SwapTestSetup {
        vending_template:
            VendingTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_pool,
        marketplace,
    } = setup_swap_test(5000).unwrap();
    let user2 = setup_addtl_account(&mut router, "asset", 100_u128).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_pool,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000_u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_pool,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        250,
        300,
    );

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    for pool in pools {
        if !pool.can_sell_nfts() {
            continue;
        }
        let nfts_to_swap_for: Vec<NftSwap> = pool
            .nft_token_ids
            .clone()
            .into_iter()
            .take(3)
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(100_000_u128),
            })
            .collect();
        let sim_msg = SimDirectSwapTokensForSpecificNfts {
            pool_id: pool.id,
            nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: true,
                asset_recipient: None,
                finder: Some(user2.to_string()),
            },
        };
        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg);

        for swap in res.unwrap().swaps {
            validate_swap_fees(
                &swap,
                &pool,
                &marketplace_params,
                &collection_info.royalty_info,
            );
        }
    }
}
