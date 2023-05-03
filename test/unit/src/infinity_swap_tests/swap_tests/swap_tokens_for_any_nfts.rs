use crate::helpers::nft_functions::mint_and_approve_many;
use crate::helpers::pool_functions::prepare_pool_variations;
use crate::helpers::swap_functions::{setup_swap_test, validate_swap_outcome, SwapTestSetup};
use crate::helpers::utils::get_native_balances;
use cosmwasm_std::{coins, Addr, Timestamp, Uint128};
use cw_multi_test::Executor;
use infinity_shared::interface::{SwapParams, SwapResponse};
use infinity_swap::msg::{ExecuteMsg, QueryMsg};
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn correct_swap_simple() {
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
        owner_token_ids.to_vec(),
        6,
        true,
        250,
        300,
    );

    let finder = Addr::unchecked("finder");
    let mut check_addresses = vec![
        accts.owner.clone(),
        accts.bidder.clone(),
        finder.clone(),
        infinity_swap.clone(),
    ];

    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    if let Some(_royalty_info) = &collection_info.royalty_info {
        check_addresses.push(Addr::unchecked(_royalty_info.payment_address.clone()));
    }

    let num_swaps: u8 = 10;
    let orders: Vec<Uint128> = vec![Uint128::from(1_000_000u128); num_swaps as usize];
    let sim_msg = QueryMsg::SimSwapTokensForAnyNfts {
        collection: collection.to_string(),
        orders: orders.clone(),
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: Some(finder.to_string()),
        },
    };

    let _res: SwapResponse = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg)
        .unwrap();

    let exec_msg = ExecuteMsg::SwapTokensForAnyNfts {
        collection: collection.to_string(),
        orders: orders.clone(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: Some(finder.to_string()),
        },
    };

    let sender_amount = orders.iter().sum::<Uint128>();

    let pre_swap_balances = get_native_balances(&router, &check_addresses);
    let exec_res = router
        .execute_contract(
            accts.bidder.clone(),
            infinity_swap.clone(),
            &exec_msg,
            &coins(sender_amount.u128(), NATIVE_DENOM),
        )
        .unwrap();
    let post_swap_balances = get_native_balances(&router, &check_addresses);

    validate_swap_outcome(
        &router,
        &exec_res,
        num_swaps,
        &pre_swap_balances,
        &post_swap_balances,
        &pools,
        &collection_info.royalty_info,
        &accts.bidder,
        &Some(finder),
    );
}
