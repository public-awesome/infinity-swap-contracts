use crate::helpers::nft_functions::mint_and_approve_many;
use crate::helpers::pool_functions::prepare_pool_variations;
use crate::helpers::swap_functions::{setup_swap_test, validate_swap_outcome, SwapTestSetup};
use crate::helpers::utils::get_native_balances;
use cosmwasm_std::{coins, Addr, Timestamp, Uint128};
use cw_multi_test::Executor;
use infinity_pool::msg::{
    ExecuteMsg, NftSwap, NftTokenIdsResponse, PoolNftSwap, QueryMsg, QueryOptions, SwapParams,
    SwapResponse,
};
use infinity_pool::state::Pool;
use itertools::Itertools;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::msg::VendingTemplateResponse;

#[test]
fn correct_swap_simple() {
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
        owner_token_ids.to_vec(),
        6,
        true,
        250,
        300,
    );

    let pool_chunks: Vec<Vec<&Pool>> = pools
        .iter()
        .filter(|&p| p.can_sell_nfts())
        .chunks(3_usize)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    let finder = Addr::unchecked("finder");
    let mut check_addresses = vec![
        accts.owner.clone(),
        accts.bidder.clone(),
        finder.clone(),
        infinity_pool.clone(),
    ];

    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    if let Some(_royalty_info) = &collection_info.royalty_info {
        check_addresses.push(Addr::unchecked(_royalty_info.payment_address.clone()));
    }

    for chunk in pool_chunks {
        let mut pool_nfts_to_swap_for: Vec<PoolNftSwap> = vec![];
        let mut sender_amount = Uint128::zero();

        let swaps_per_chunk: u8 = 3;
        for pool in &chunk {
            let nft_token_ids_response: NftTokenIdsResponse = router
                .wrap()
                .query_wasm_smart(
                    infinity_pool.clone(),
                    &QueryMsg::PoolNftTokenIds {
                        pool_id: pool.id,
                        query_options: QueryOptions {
                            descending: None,
                            start_after: None,
                            limit: Some(swaps_per_chunk as u32),
                        },
                    },
                )
                .unwrap();

            pool_nfts_to_swap_for.push(PoolNftSwap {
                pool_id: pool.id,
                nft_swaps: nft_token_ids_response
                    .nft_token_ids
                    .iter()
                    .map(|token_id| {
                        let nft_swap = NftSwap {
                            nft_token_id: token_id.to_string(),
                            token_amount: Uint128::from(100_000u128),
                        };
                        sender_amount += nft_swap.token_amount;
                        nft_swap
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
                finder: Some(finder.to_string()),
            },
        };

        let _sim_res: SwapResponse = router
            .wrap()
            .query_wasm_smart(infinity_pool.clone(), &sim_msg)
            .unwrap();

        let exec_msg = ExecuteMsg::SwapTokensForSpecificNfts {
            collection: collection.to_string(),
            pool_nfts_to_swap_for: pool_nfts_to_swap_for.clone(),
            swap_params: SwapParams {
                deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
                robust: false,
                asset_recipient: None,
                finder: Some(finder.to_string()),
            },
        };

        let pre_swap_balances = get_native_balances(&router, &check_addresses);
        let exec_res = router
            .execute_contract(
                accts.bidder.clone(),
                infinity_pool.clone(),
                &exec_msg,
                &coins(sender_amount.u128(), NATIVE_DENOM),
            )
            .unwrap();
        let post_swap_balances = get_native_balances(&router, &check_addresses);

        validate_swap_outcome(
            &router,
            &exec_res,
            chunk.len() as u8 * swaps_per_chunk,
            &pre_swap_balances,
            &post_swap_balances,
            &pools,
            &collection_info.royalty_info,
            &accts.bidder,
            &Some(finder.clone()),
        );
    }
}
