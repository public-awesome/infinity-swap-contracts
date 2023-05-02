use crate::helpers::marketplace_functions::{set_bid, set_collection_bid};
use crate::helpers::nft_functions::{mint_and_approve_many, validate_nft_owner};
use crate::helpers::swap_functions::{setup_swap_test, SwapTestSetup};
use crate::setup::msg::MarketAccounts;
use crate::setup::setup_accounts::{setup_addtl_account, INITIAL_BALANCE};
use crate::setup::setup_marketplace::MIN_EXPIRY;
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;
use infinity_marketplace_adapter::msg::{
    ExecuteMsg as InfinityAdapterExecuteMsg, QueryMsg as InfinityAdapterQueryMsg,
};
use infinity_shared::interface::{NftOrder, SwapParams, SwapResponse};
use sg_marketplace::state::SaleType;
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn swap_nft_for_token_marketplace_adapter() {
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
        &infinity_marketplace_adapter,
        5,
    );

    /*
        Testing Bid Matching
            * Test CollectionBids are taken when no Bid is found
            * Test Bids are taken when greater than or equal to CollectionBid
            * Test CollectionBids are taken when greater than Bid
            * Test Bids are taken when no CollectionBid is found
            * Test Bids are not accepted when below NftOrder

        Bids
            bid_0: token_id_0, None
            bid_1: token_id_1, 1002 STARS
            bid_2: token_id_2, 999 STARS
            bid_3: token_id_3, 998 STARS
            bid_3: token_id_4, 997 STARS

        Collection Bids
            collection_bid_0: 1001 STARS
            collection_bid_1: 1000 STARS

        Match Order
            token_id_0: collection_bid_0
            token_id_1: bid_1
            token_id_2: collection_bid_1
            token_id_3: bid_3
            token_id_4: None
    */

    // Setup Bids
    let expires = block_time.plus_seconds(MIN_EXPIRY + 1);
    for (idx, token_id) in owner_token_ids.iter().enumerate() {
        let bid_amount = match idx {
            0 => None,
            1 => Some(1002u128),
            2 => Some(999u128),
            3 => Some(998u128),
            4 => Some(997u128),
            _ => unreachable!(),
        };
        if let Some(_bid_amount) = bid_amount {
            set_bid(
                &mut router,
                marketplace.clone(),
                bidder.clone(),
                _bid_amount,
                collection.to_string(),
                token_id.parse::<u32>().unwrap(),
                expires,
                SaleType::FixedPrice,
                None,
                Some(250),
            );
        }
    }

    // Setup CollectionBids
    let collection_bidder_0 =
        setup_addtl_account(&mut router, "collection_bidder_0", INITIAL_BALANCE).unwrap();
    set_collection_bid(
        &mut router,
        marketplace.clone(),
        collection_bidder_0,
        collection.to_string(),
        expires,
        Some(250),
        1001u128,
    );
    let collection_bidder_1 =
        setup_addtl_account(&mut router, "collection_bidder_1", INITIAL_BALANCE).unwrap();
    set_collection_bid(
        &mut router,
        marketplace,
        collection_bidder_1,
        collection.to_string(),
        expires,
        Some(250),
        1000u128,
    );

    let sim_msg = InfinityAdapterQueryMsg::SimSwapNftsForTokens {
        sender: owner.to_string(),
        collection: collection.to_string(),
        nft_orders: owner_token_ids
            .iter()
            .map(|token_id| NftOrder {
                token_id: token_id.to_string(),
                amount: Uint128::from(998u128),
            })
            .collect(),
        swap_params: SwapParams {
            deadline: expires,
            robust: true,
            asset_recipient: None,
            finder: Some(finder.to_string()),
        },
    };
    let sim_response: SwapResponse = router
        .wrap()
        .query_wasm_smart(&infinity_marketplace_adapter, &sim_msg)
        .unwrap();
    assert_eq!(sim_response.swaps.len(), 4);

    let exec_msg = InfinityAdapterExecuteMsg::SwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders: owner_token_ids
            .iter()
            .map(|token_id| NftOrder {
                token_id: token_id.to_string(),
                amount: Uint128::from(998u128),
            })
            .collect(),
        swap_params: SwapParams {
            deadline: expires,
            robust: true,
            asset_recipient: None,
            finder: Some(finder.to_string()),
        },
    };
    let exec_res = router
        .execute_contract(owner.clone(), infinity_marketplace_adapter, &exec_msg, &[])
        .unwrap();

    // Account for 4 swaps
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
        - Uint128::from(INITIAL_BALANCE);
    assert_eq!(sim_seller_amount, actual_seller_amount);

    // Verify that bidders received their NFTs
    for swap in sim_response.swaps {
        let nft_recipient = swap.nft_payments[0].address.clone();
        validate_nft_owner(
            &router,
            &collection,
            swap.nft_payments[0].token_id.clone(),
            &Addr::unchecked(nft_recipient),
        );
    }
}
