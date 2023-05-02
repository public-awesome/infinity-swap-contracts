use crate::helpers::marketplace_functions::{set_bid, set_collection_bid};
use crate::helpers::nft_functions::mint_and_approve_many;
use crate::helpers::swap_functions::{setup_swap_test, SwapTestSetup};
use crate::setup::msg::MarketAccounts;
use crate::setup::setup_accounts::{setup_addtl_account, INITIAL_BALANCE};
use crate::setup::setup_marketplace::MIN_EXPIRY;
use cosmwasm_std::Uint128;
use infinity_marketplace_adapter::msg::QueryMsg as InfinityAdapterQueryMsg;
use infinity_shared::interface::{
    NftOrder, NftPayment, Swap, SwapParams, SwapResponse, TokenPayment, TransactionType,
};
use sg_marketplace::state::SaleType;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn sim_nft_for_token_marketplace_adapter() {
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
                None,
            );
        }
    }

    // Setup CollectionBids
    let collection_bidder_0 =
        setup_addtl_account(&mut router, "collection_bidder_0", INITIAL_BALANCE).unwrap();
    set_collection_bid(
        &mut router,
        marketplace.clone(),
        collection_bidder_0.clone(),
        collection.to_string(),
        expires,
        None,
        1001u128,
    );
    let collection_bidder_1 =
        setup_addtl_account(&mut router, "collection_bidder_1", INITIAL_BALANCE).unwrap();
    set_collection_bid(
        &mut router,
        marketplace.clone(),
        collection_bidder_1.clone(),
        collection.to_string(),
        expires,
        None,
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
            finder: None,
        },
    };
    let response: SwapResponse = router
        .wrap()
        .query_wasm_smart(infinity_marketplace_adapter.clone(), &sim_msg)
        .unwrap();

    assert_eq!(response.swaps.len(), 4);

    let swap_0 = response.swaps[0].clone();
    assert_eq!(
        swap_0,
        Swap {
            source: marketplace.to_string(),
            transaction_type: TransactionType::UserSubmitsNfts,
            sale_price: Uint128::from(1001u128),
            network_fee: Uint128::from(20u128),
            nft_payments: vec![NftPayment {
                label: "buyer".to_string(),
                token_id: owner_token_ids[0].to_string(),
                address: collection_bidder_0.to_string()
            }],
            token_payments: vec![
                TokenPayment {
                    label: "royalty".to_string(),
                    amount: Uint128::from(100u128),
                    address: creator.to_string()
                },
                TokenPayment {
                    label: "seller".to_string(),
                    amount: Uint128::from(881u128),
                    address: owner.to_string()
                },
            ]
        }
    );

    let swap_1 = response.swaps[1].clone();
    assert_eq!(swap_1.sale_price, Uint128::from(1002u128));
    assert_eq!(
        swap_1.nft_payments[0].token_id,
        owner_token_ids[1].to_string()
    );
    assert_eq!(swap_1.nft_payments[0].address, bidder.to_string());

    let swap_2 = response.swaps[2].clone();
    assert_eq!(swap_2.sale_price, Uint128::from(1000u128));
    assert_eq!(
        swap_2.nft_payments[0].token_id,
        owner_token_ids[2].to_string()
    );
    assert_eq!(
        swap_2.nft_payments[0].address,
        collection_bidder_1.to_string()
    );

    let swap_3 = response.swaps[3].clone();
    assert_eq!(swap_3.sale_price, Uint128::from(998u128));
    assert_eq!(
        swap_3.nft_payments[0].token_id,
        owner_token_ids[3].to_string()
    );
    assert_eq!(swap_3.nft_payments[0].address, bidder.to_string());
}
