use crate::msg::TransactionType;
use crate::state::Pool;
use crate::swap_processor::Swap;
use crate::testing::setup::msg::MarketAccounts;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;

use anyhow::Error;
use cosmwasm_std::{Addr, Decimal, Uint128};
use sg721::RoyaltyInfoResponse;
use sg_marketplace::msg::ParamsResponse;
use sg_std::GENESIS_MINT_START_TIME;
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

pub fn validate_swap(
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

    if pool.reinvest_tokens && swap.transaction_type == TransactionType::Buy {
        assert_eq!(swap.seller_payment, None);
    } else {
        assert_eq!(
            swap.seller_payment.as_ref().unwrap().amount,
            remaining_payment
        );
    }

    if pool.reinvest_nfts && swap.transaction_type == TransactionType::Sell {
        assert_eq!(swap.nft_payment, None);
    } else {
        assert!(swap.nft_payment.as_ref().unwrap().nft_token_id.len() > 0);
    }
}
