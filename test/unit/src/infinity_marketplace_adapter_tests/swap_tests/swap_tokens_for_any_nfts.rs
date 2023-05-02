use crate::helpers::marketplace_functions::set_ask;
use crate::helpers::nft_functions::{approve, mint_and_approve_many, validate_nft_owner};
use crate::helpers::swap_functions::{setup_swap_test, SwapTestSetup};
use crate::setup::msg::MarketAccounts;
use crate::setup::setup_accounts::{setup_addtl_account, INITIAL_BALANCE};
use crate::setup::setup_marketplace::MIN_EXPIRY;
use cosmwasm_std::{coins, Uint128};
use cw_multi_test::Executor;
use infinity_marketplace_adapter::msg::{
    ExecuteMsg as InfinityAdapterExecuteMsg, QueryMsg as InfinityAdapterQueryMsg,
};
use infinity_shared::interface::{SwapParams, SwapResponse};
use sg_marketplace::state::SaleType;
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn swap_token_for_any_nfts_marketplace_adapter() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts:
                    MarketAccounts {
                        creator,
                        bidder,
                        owner,
                    },
                collection_response_vec,
                ..
            },
        marketplace,
        infinity_marketplace_adapter,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();
    let block_time = router.block_info().time;

    let finder = setup_addtl_account(&mut router, "finder", INITIAL_BALANCE).unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &creator,
        &owner,
        &minter,
        &collection,
        &marketplace,
        4,
    );

    /*
        Test Ask Matching:
            * Test that only Asks below or equal to the desired price are taken
            * Test that they are taken in the order of price ascending
            * Test that multiple orders can be filled

        Asks
            ask_0: token_id_0, 999 STARS
            ask_1: token_id_1, 1001 STARS
            ask_2: token_id_2, None
            ask_3: token_id_3, 1000 STARS

        Match Order
            token_id_0: ask_0
            token_id_3: ask_3
    */

    // Setup Asks
    let expires = block_time.plus_seconds(MIN_EXPIRY + 1);
    for (idx, token_id) in owner_token_ids.iter().enumerate() {
        let token_id: u32 = token_id.parse().unwrap();
        approve(&mut router, &owner, &collection, &marketplace, token_id);

        let ask_amount = match idx {
            0 => Some(999u128),
            1 => Some(1001u128),
            2 => None,
            3 => Some(1000u128),
            _ => unreachable!(),
        };
        if let Some(_ask_amount) = ask_amount {
            set_ask(
                &mut router,
                marketplace.clone(),
                owner.clone(),
                _ask_amount,
                collection.to_string(),
                token_id,
                expires,
                SaleType::FixedPrice,
                Some(250),
                None,
                None,
            );
        }
    }
    let seller_balance = router
        .wrap()
        .query_balance(&owner, NATIVE_DENOM)
        .unwrap()
        .amount;

    let nft_orders: Vec<Uint128> = owner_token_ids
        .iter()
        .map(|_| Uint128::from(1000u128))
        .collect();
    let fund_amount = nft_orders
        .iter()
        .fold(Uint128::zero(), |acc, order| acc + order);

    let sim_msg = InfinityAdapterQueryMsg::SimSwapTokensForAnyNfts {
        sender: bidder.to_string(),
        collection: collection.to_string(),
        nft_orders: nft_orders.clone(),
        swap_params: SwapParams {
            deadline: expires,
            robust: true,
            asset_recipient: None,
            finder: Some(finder.to_string()),
        },
    };
    let sim_response: SwapResponse = router
        .wrap()
        .query_wasm_smart(infinity_marketplace_adapter.clone(), &sim_msg)
        .unwrap();

    assert_eq!(sim_response.swaps.len(), 2);

    let exec_msg = InfinityAdapterExecuteMsg::SwapTokensForAnyNfts {
        collection: collection.to_string(),
        nft_orders,
        swap_params: SwapParams {
            deadline: expires,
            robust: true,
            asset_recipient: None,
            finder: Some(finder.to_string()),
        },
    };
    let exec_res = router
        .execute_contract(
            bidder.clone(),
            infinity_marketplace_adapter.clone(),
            &exec_msg,
            &coins(fund_amount.u128(), NATIVE_DENOM),
        )
        .unwrap();

    // Account for 2 swaps
    // -------------------

    // Verify burn amount
    let sim_burn_amount = sim_response
        .swaps
        .iter()
        .fold(Uint128::zero(), |acc, swap| acc + swap.network_fee);

    let actual_burn_amount = exec_res
        .events
        .iter()
        .filter(|&event| event.ty == "wasm-fair-burn")
        .fold(Uint128::zero(), |acc, event| {
            let burn_amount = event
                .attributes
                .iter()
                .find(|attr| attr.key == "burn_amount")
                .unwrap()
                .value
                .parse::<u64>()
                .unwrap();
            let dist_amount = event
                .attributes
                .iter()
                .find(|attr| attr.key == "dist_amount")
                .unwrap()
                .value
                .parse::<u64>()
                .unwrap();
            acc + Uint128::from(burn_amount + dist_amount)
        });
    assert_eq!(sim_burn_amount, actual_burn_amount);

    // Verify royalties paid
    let sim_royalty_amount = sim_response
        .swaps
        .iter()
        .fold(Uint128::zero(), |acc, swap| {
            let royalty_payment = swap
                .token_payments
                .iter()
                .find(|&payment| payment.label == "royalty")
                .unwrap()
                .amount;
            acc + royalty_payment
        });

    let actual_royalty_amount = exec_res
        .events
        .iter()
        .filter(|&event| event.ty == "wasm-royalty-payout")
        .fold(Uint128::zero(), |acc, event| {
            let amount = event
                .attributes
                .iter()
                .find(|attr| attr.key == "amount")
                .unwrap()
                .value
                .parse::<u64>()
                .unwrap();
            acc + Uint128::from(amount)
        });
    assert_eq!(sim_royalty_amount, actual_royalty_amount);

    // Verify finders paid
    let sim_finder_amount = sim_response
        .swaps
        .iter()
        .fold(Uint128::zero(), |acc, swap| {
            let finder_payment = swap
                .token_payments
                .iter()
                .find(|&payment| payment.label == "finder")
                .unwrap()
                .amount;
            acc + finder_payment
        });

    let actual_finder_amount = router
        .wrap()
        .query_balance(&finder, NATIVE_DENOM)
        .unwrap()
        .amount
        - Uint128::from(INITIAL_BALANCE);
    assert_eq!(sim_finder_amount, actual_finder_amount);

    // Verify that seller was paid
    let sim_seller_amount = sim_response
        .swaps
        .iter()
        .fold(Uint128::zero(), |acc, swap| {
            let seller_payment = swap
                .token_payments
                .iter()
                .find(|&payment| payment.label == "seller")
                .unwrap()
                .amount;
            acc + seller_payment
        });
    let actual_seller_amount = router
        .wrap()
        .query_balance(&owner, NATIVE_DENOM)
        .unwrap()
        .amount
        - seller_balance;
    assert_eq!(sim_seller_amount, actual_seller_amount);

    // Verify that bidders received their NFTs
    for swap in sim_response.swaps {
        validate_nft_owner(
            &router,
            &collection,
            swap.nft_payments[0].token_id.clone(),
            &bidder,
        );
    }
}
