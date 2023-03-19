use crate::helpers::nft_functions::validate_nft_owner;
use crate::setup::msg::MarketAccounts;
use crate::setup::setup_infinity_pool::setup_infinity_pool;
use crate::setup::setup_marketplace::setup_marketplace;
use crate::setup::templates::standard_minter_template;
use infinity_pool::msg::{PoolsByIdResponse, QueryMsg, TransactionType};
use infinity_pool::state::{Pool, PoolType};
use infinity_pool::swap_processor::Swap;

use anyhow::Error;
use cosmwasm_std::{Addr, Coin, Decimal, Event, Uint128};
use cw_multi_test::AppResponse;
use sg721::RoyaltyInfoResponse;
use sg_marketplace::msg::ParamsResponse;
use sg_multi_test::StargazeApp;
use sg_std::GENESIS_MINT_START_TIME;
use std::collections::{HashMap, HashSet};
use test_suite::common_setup::msg::VendingTemplateResponse;
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

pub struct SwapTestSetup {
    pub marketplace: Addr,
    pub infinity_pool: Addr,
    pub vending_template: VendingTemplateResponse<MarketAccounts>,
}

pub fn setup_swap_test(num_tokens: u32) -> Result<SwapTestSetup, Error> {
    let mut vt = standard_minter_template(num_tokens);

    let marketplace = setup_marketplace(&mut vt.router, vt.accts.creator.clone()).unwrap();
    setup_block_time(&mut vt.router, GENESIS_MINT_START_TIME, None);

    let infinity_pool = setup_infinity_pool(
        &mut vt.router,
        vt.accts.creator.clone(),
        marketplace.clone(),
    )
    .unwrap();

    Ok(SwapTestSetup {
        marketplace,
        infinity_pool,
        vending_template: vt,
    })
}

fn validate_address_paid(
    pre_swap_balances: &HashMap<Addr, Coin>,
    post_swap_balances: &HashMap<Addr, Coin>,
    check_addr: &Addr,
) {
    assert!(pre_swap_balances[check_addr].amount < post_swap_balances[check_addr].amount);
}

pub fn validate_swap_fees(
    swap: &Swap,
    pool: &Pool,
    marketplace_params: &ParamsResponse,
    royalty_info: &Option<RoyaltyInfoResponse>,
) {
    assert_eq!(swap.pool_id, pool.id);

    assert_eq!(
        swap.network_fee,
        swap.spot_price * marketplace_params.params.trading_fee_percent / Uint128::from(100u128),
    );
    let mut remaining_payment = swap.spot_price - swap.network_fee;

    if royalty_info.is_none() {
        assert_eq!(swap.royalty_payment, None);
    } else {
        let royalty_info = royalty_info.as_ref().unwrap();
        if royalty_info.share > Decimal::zero() {
            assert_eq!(
                swap.royalty_payment.as_ref().unwrap().amount,
                swap.spot_price * royalty_info.share,
            );
            assert_eq!(
                swap.royalty_payment.as_ref().unwrap().address,
                royalty_info.payment_address
            );
            remaining_payment -= swap.royalty_payment.as_ref().unwrap().amount;
        } else {
            assert_eq!(swap.royalty_payment, None);
        }
    }

    if pool.finders_fee_percent > Decimal::zero() {
        assert_eq!(
            swap.finder_payment.as_ref().unwrap().amount,
            swap.spot_price * pool.finders_fee_percent / Uint128::from(100u128),
        );
        remaining_payment -= swap.finder_payment.as_ref().unwrap().amount;
    } else {
        assert_eq!(swap.finder_payment, None);
    }

    if pool.reinvest_tokens && swap.transaction_type == TransactionType::TokensForNfts {
        assert_eq!(swap.seller_payment, None);
    } else {
        assert_eq!(
            swap.seller_payment.as_ref().unwrap().amount,
            remaining_payment
        );
    }

    assert!(!swap.nft_payment.nft_token_id.is_empty());
}

pub fn validate_swap_outcome(
    router: &StargazeApp,
    response: &AppResponse,
    expected_num_swaps: u8,
    pre_swap_balances: &HashMap<Addr, Coin>,
    post_swap_balances: &HashMap<Addr, Coin>,
    pools: &Vec<Pool>,
    royalty_info: &Option<RoyaltyInfoResponse>,
    sender: &Addr,
    finder: &Option<Addr>,
) {
    // Test that addresses receive assets
    // Network fee was paid
    response.assert_event(&Event::new("wasm-fair-burn"));

    let mut num_swaps = 0u8;
    let mut expected_pool_quote_updates = 0u8;
    let mut pool_ids: HashSet<u64> = HashSet::new();

    let wasm_swap_event = response
        .events
        .iter()
        .find(|&e| e.ty == "wasm-swap")
        .unwrap();

    let infinity_pool_addr = Addr::unchecked(wasm_swap_event.attributes[0].value.clone());
    let tx_type = wasm_swap_event
        .attributes
        .iter()
        .find(|&a| a.key == "transaction_type")
        .unwrap()
        .value
        .clone();

    for event in response.events.iter() {
        if event.ty == "wasm-swap" {
            num_swaps += 1;

            let pool_id = event
                .attributes
                .iter()
                .find(|&a| a.key == "pool_id")
                .unwrap()
                .value
                .parse::<u64>()
                .unwrap();

            let pool = pools.iter().find(|&p| p.id == pool_id).unwrap();

            if !pool_ids.contains(&pool_id) {
                expected_pool_quote_updates += 1;
                if pool.pool_type == PoolType::Trade {
                    expected_pool_quote_updates += 1;
                }
                pool_ids.insert(pool_id);
            }

            let token_id = event
                .attributes
                .iter()
                .find(|&a| a.key == "nft_payment_token_id")
                .unwrap()
                .value
                .clone();

            // Verify that finder was paid
            if pool.finders_fee_percent > Decimal::zero() && finder.is_some() {
                validate_address_paid(
                    pre_swap_balances,
                    post_swap_balances,
                    finder.as_ref().unwrap(),
                );
            }

            if tx_type == "NftsForTokens" {
                // Verify pool owner received NFT
                let pool_owner_account = if pool.pool_type == PoolType::Trade && pool.reinvest_nfts
                {
                    infinity_pool_addr.clone()
                } else {
                    pool.get_recipient()
                };
                validate_nft_owner(router, &pool.collection, token_id, &pool_owner_account);

                // Verify user received tokens
                validate_address_paid(pre_swap_balances, post_swap_balances, sender);
            } else {
                // Verify pool owner received tokens
                let pool_owner_account =
                    if pool.pool_type == PoolType::Trade && pool.reinvest_tokens {
                        infinity_pool_addr.clone()
                    } else {
                        pool.get_recipient()
                    };
                validate_address_paid(pre_swap_balances, post_swap_balances, &pool_owner_account);

                // Verify user received NFT
                validate_nft_owner(router, &pool.collection, token_id, sender);
            }
        }
    }

    // Verify that swap events are emitted
    assert_eq!(num_swaps, expected_num_swaps);

    // Verify royalty was paid
    if let Some(_royalty_info) = royalty_info {
        validate_address_paid(
            pre_swap_balances,
            post_swap_balances,
            &Addr::unchecked(&_royalty_info.payment_address),
        );
    }

    // Verify that pool state update events are emitted
    let num_pool_updates = response
        .events
        .iter()
        .filter(|&e| e.ty == "wasm-pool-swap-update")
        .count();
    assert_eq!(num_pool_updates, pool_ids.len());

    // Verify that new pool quotes are generated
    let num_pool_quote_updates = response
        .events
        .iter()
        .filter(|&e| {
            [
                "wasm-add-buy-pool-quote".to_string(),
                "wasm-remove-buy-pool-quote".to_string(),
                "wasm-add-sell-pool-quote".to_string(),
                "wasm-remove-sell-pool-quote".to_string(),
            ]
            .contains(&e.ty)
        })
        .count();
    assert_eq!(num_pool_quote_updates, expected_pool_quote_updates as usize);

    // Verify that pool state is updated
    let get_pools_msg = QueryMsg::PoolsById {
        pool_ids: pool_ids.clone().into_iter().collect(),
    };
    let post_swap_pools: PoolsByIdResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool_addr, &get_pools_msg)
        .unwrap();
    let mut post_swap_pools_map: HashMap<u64, Pool> = HashMap::new();
    for (pool_id, pool) in post_swap_pools.pools {
        post_swap_pools_map.insert(pool_id, pool.unwrap());
    }

    for pool in pools {
        if !pool_ids.contains(&pool.id) {
            continue;
        }
        let post_swap_pool = post_swap_pools_map.get(&pool.id).unwrap();
        if tx_type == "NftsForTokens" {
            assert!(pool.spot_price >= post_swap_pool.spot_price)
        } else {
            assert!(pool.spot_price <= post_swap_pool.spot_price)
        }
    }
}
