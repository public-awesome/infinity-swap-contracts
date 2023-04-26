use crate::helpers::nft_functions::mint_and_approve_many;
use crate::helpers::pool_functions::prepare_pool_variations;
use crate::helpers::swap_functions::{setup_swap_test, validate_swap_fees, SwapTestSetup};
use crate::setup::setup_accounts::setup_addtl_account;
use crate::setup::setup_infinity_swap::setup_infinity_swap;
use crate::setup::setup_marketplace::setup_marketplace;
use crate::setup::templates::minter_two_collections;
use cosmwasm_std::{StdError, StdResult, Timestamp, Uint128};
use infinity_swap::msg::{
    NftSwap, NftTokenIdsResponse, PoolNftSwap, QueryMsg, QueryOptions, SwapParams, SwapResponse,
};
use infinity_swap::state::Pool;
use itertools::Itertools;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::GENESIS_MINT_START_TIME;
use std::vec;
use test_suite::common_setup::msg::MinterTemplateResponse;
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

#[test]
fn cant_swap_inactive_pool() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
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
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        false,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .into_iter()
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(100_000u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for,
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);
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
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
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
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        false,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| !p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .into_iter()
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(100_000u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for,
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);
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
fn cant_swap_for_pools_outside_of_collection() {
    let mut vt = minter_two_collections(5000);

    let marketplace = setup_marketplace(&mut vt.router, vt.accts.creator.clone()).unwrap();
    setup_block_time(&mut vt.router, GENESIS_MINT_START_TIME, None);

    let infinity_swap =
        setup_infinity_swap(&mut vt.router, vt.accts.creator.clone(), marketplace).unwrap();

    let MinterTemplateResponse {
        mut router,
        accts,
        collection_response_vec,
        ..
    } = vt;

    let minter = &collection_response_vec[0].minter.clone().unwrap();
    let collection_a = &collection_response_vec[0].collection.clone().unwrap();
    let collection_resp_b = &collection_response_vec[1];
    let collection_b = collection_resp_b.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        minter,
        collection_a,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        collection_a,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .into_iter()
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(100_000u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection_b.to_string(),
            pool_nfts_to_swap_for,
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);

        assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: "Querier contract error: Generic error: Invalid pool: pool does not belong to this collection"
                    .to_string()
            }
        );
    }
}

#[test]
fn can_swap_active_pool() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
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
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .into_iter()
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(100_000u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for,
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);

        assert!(res.is_ok());
        assert!(!res.unwrap().swaps.is_empty());
    }
}

#[test]
fn incorrect_nfts_error() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
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
        &infinity_swap,
        100,
    );
    let mut bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk {
            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: bidder_token_ids
                    .drain(0..3)
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(100_000u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for: pool_nfts_to_swap_for.clone(),
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);

        assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: format!(
                    "Querier contract error: Generic error: Swap error: pool {} does not own NFT {}",
                    pool_nfts_to_swap_for[0].pool_id, pool_nfts_to_swap_for[0].nft_swaps[0].nft_token_id
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
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
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
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .into_iter()
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(10u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for,
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);

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
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
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
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            let mut pool_nft_token_ids: Vec<String> =
                nft_token_ids_response.nft_token_ids.into_iter().collect();
            let mut pool_nft_swap = PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: pool_nft_token_ids
                    .drain(0..2_usize)
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(100_000_u128),
                    })
                    .collect(),
            };
            pool_nft_swap.nft_swaps.extend(
                pool_nft_token_ids
                    .drain(0..1_usize)
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id,
                        token_amount: Uint128::from(10u128),
                    })
                    .collect::<Vec<NftSwap>>(),
            );
            pool_nfts_to_swap_for.push(pool_nft_swap);
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for,
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);

        assert!(res.is_ok());
        assert!(!res.unwrap().swaps.is_empty());
    }
}

#[test]
fn trading_fee_is_applied_correctly() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        marketplace,
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
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        0,
        0,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk.iter() {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .iter()
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id.to_string(),
                        token_amount: Uint128::from(100_000u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for,
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
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);

        for swap in res.unwrap().swaps {
            let pool = chunk.iter().find(|p| p.id == swap.pool_id).unwrap();
            validate_swap_fees(
                &swap,
                pool,
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
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        marketplace,
        ..
    } = setup_swap_test(5000).unwrap();
    let user2 = setup_addtl_account(&mut router, "asset", 100u128).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids,
        6,
        true,
        250,
        300,
    );

    let pool_chunks: Vec<Vec<Pool>> = pools
        .into_iter()
        .filter(|p| p.can_sell_nfts())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        for pool in chunk.iter() {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_swap.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(3),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .iter()
                    .map(|token_id| NftSwap {
                        nft_token_id: token_id.to_string(),
                        token_amount: Uint128::from(100_000u128),
                    })
                    .collect(),
            });
        }

        let sim_msg = QueryMsg::SimSwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for,
            sender: accts.bidder.to_string(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: false,
                asset_recipient: None,
                finder: Some(user2.to_string()),
            },
        };

        let res: StdResult<SwapResponse> = router
            .wrap()
            .query_wasm_smart(infinity_swap.clone(), &sim_msg);

        for swap in res.unwrap().swaps {
            let pool = chunk.iter().find(|p| p.id == swap.pool_id).unwrap();
            validate_swap_fees(
                &swap,
                pool,
                &marketplace_params,
                &collection_info.royalty_info,
            );
        }
    }
}
