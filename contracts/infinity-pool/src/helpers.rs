use crate::{pool::Pool, state::POOL_CONFIG, ContractError};

use cosmwasm_std::{
    coin, to_binary, Addr, Decimal, QuerierWrapper, StdError, Storage, Uint128, WasmMsg,
};
use infinity_index::msg::ExecuteMsg as InfinityIndexExecuteMsg;
use sg1::fair_burn;
use sg721::RoyaltyInfo;
use sg_marketplace_common::transfer_coin;
use sg_std::{Response, NATIVE_DENOM};

pub fn load_pool(
    contract: &Addr,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
) -> Result<Pool, ContractError> {
    let pool_config = POOL_CONFIG.load(storage)?;
    let total_tokens = querier.query_balance(contract, NATIVE_DENOM)?.amount;
    Ok(Pool::new(pool_config, total_tokens))
}

pub fn update_sell_to_pool_quote(
    infinity_index: &Addr,
    collection: &Addr,
    quote_price: Option<Uint128>,
    response: Response,
) -> Response {
    response.add_message(WasmMsg::Execute {
        contract_addr: infinity_index.to_string(),
        msg: to_binary(&InfinityIndexExecuteMsg::UpdateSellToPoolQuote {
            collection: collection.to_string(),
            quote_price,
        })
        .unwrap(),
        funds: vec![],
    })
}

pub fn update_buy_from_pool_quote(
    infinity_index: &Addr,
    collection: &Addr,
    quote_price: Option<Uint128>,
    response: Response,
) -> Response {
    response.add_message(WasmMsg::Execute {
        contract_addr: infinity_index.to_string(),
        msg: to_binary(&InfinityIndexExecuteMsg::UpdateBuyFromPoolQuote {
            collection: collection.to_string(),
            quote_price,
        })
        .unwrap(),
        funds: vec![],
    })
}

pub fn save_pool_and_update_indices(
    storage: &mut dyn Storage,
    pool: &mut Pool,
    infinity_index: &Addr,
    min_price: Uint128,
    mut response: Response,
) -> Result<Response, ContractError> {
    POOL_CONFIG.save(storage, &pool.config)?;

    let mut next_sell_to_pool_quote: Option<Uint128> = None;
    let mut next_buy_from_pool_quote: Option<Uint128> = None;

    if pool.config.is_active {
        if pool.can_escrow_tokens() {
            next_sell_to_pool_quote = pool.get_sell_to_pool_quote(min_price).ok();
        }
        if pool.can_escrow_nfts() {
            next_buy_from_pool_quote = pool.get_buy_from_pool_quote(min_price).ok();
        }
    }

    if pool.can_escrow_tokens() {
        response = update_sell_to_pool_quote(
            infinity_index,
            &pool.config.collection,
            next_sell_to_pool_quote,
            response,
        );
    }

    if pool.can_escrow_nfts() {
        response = update_buy_from_pool_quote(
            infinity_index,
            &pool.config.collection,
            next_buy_from_pool_quote,
            response,
        );
    }

    pool.save(storage)?;

    Ok(response)
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenPayment {
    pub amount: Uint128,
    pub recipient: Addr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransactionFees {
    pub fair_burn_fee: Uint128,
    pub seller_payment: TokenPayment,
    pub finders_fee: Option<TokenPayment>,
    pub royalty_fee: Option<TokenPayment>,
    pub swap_fee: Option<TokenPayment>,
}

/// Calculate fees for an NFT sale
pub fn calculate_nft_sale_fees(
    sale_price: Uint128,
    trading_fee_percent: Decimal,
    seller: Addr,
    finder: Option<Addr>,
    finders_fee_percent: Decimal,
    royalty_info: Option<RoyaltyInfo>,
    swap_fee_percent: Decimal,
    swap_fee_recipient: Option<Addr>,
) -> Result<TransactionFees, ContractError> {
    // Calculate Fair Burn
    let fair_burn_fee = sale_price * trading_fee_percent / Uint128::from(100u128);

    let mut seller_amount =
        sale_price.checked_sub(fair_burn_fee).map_err(Into::<StdError>::into)?;

    // Calculate finders fee
    let mut finders_fee: Option<TokenPayment> = None;
    if let Some(finder) = finder {
        let finders_fee_amount = sale_price * finders_fee_percent / Uint128::from(100u128);

        if finders_fee_amount > Uint128::zero() {
            finders_fee = Some(TokenPayment {
                amount: finders_fee_amount,
                recipient: finder,
            });
            seller_amount = seller_amount
                .checked_sub(Uint128::from(finders_fee_amount))
                .map_err(Into::<StdError>::into)?;
        }
    };

    // Calculate royalty
    let mut royalty_fee: Option<TokenPayment> = None;
    if let Some(royalty_info) = royalty_info {
        let royalty_fee_amount = sale_price * royalty_info.share;
        if royalty_fee_amount > Uint128::zero() {
            royalty_fee = Some(TokenPayment {
                amount: royalty_fee_amount,
                recipient: royalty_info.payment_address,
            });
            seller_amount = seller_amount
                .checked_sub(Uint128::from(royalty_fee_amount))
                .map_err(Into::<StdError>::into)?;
        }
    };

    // Calculate swap fee paid to liquidity provider
    let mut swap_fee: Option<TokenPayment> = None;
    if let Some(swap_fee_recipient) = swap_fee_recipient {
        let swap_fee_amount = sale_price * swap_fee_percent / Uint128::from(100u128);

        if swap_fee_amount > Uint128::zero() {
            swap_fee = Some(TokenPayment {
                amount: swap_fee_amount,
                recipient: swap_fee_recipient,
            });
            seller_amount = seller_amount
                .checked_sub(Uint128::from(swap_fee_amount))
                .map_err(Into::<StdError>::into)?;
        }
    };

    // Pay seller
    let seller_payment = TokenPayment {
        amount: seller_amount,
        recipient: seller,
    };

    Ok(TransactionFees {
        fair_burn_fee,
        seller_payment,
        finders_fee,
        royalty_fee,
        swap_fee,
    })
}

pub fn pay_out_nft_sale_fees(
    mut response: Response,
    tx_fees: TransactionFees,
    developer: Option<Addr>,
) -> Result<Response, ContractError> {
    fair_burn(tx_fees.fair_burn_fee.u128(), developer, &mut response);

    if let Some(finders_fee) = &tx_fees.finders_fee {
        response = response.add_submessage(transfer_coin(
            coin(finders_fee.amount.u128(), NATIVE_DENOM),
            &finders_fee.recipient,
        ));
    }

    if let Some(royalty_fee) = &tx_fees.royalty_fee {
        response = response.add_submessage(transfer_coin(
            coin(royalty_fee.amount.u128(), NATIVE_DENOM),
            &royalty_fee.recipient,
        ));
    }

    if let Some(swap_fee) = &tx_fees.swap_fee {
        response = response.add_submessage(transfer_coin(
            coin(swap_fee.amount.u128(), NATIVE_DENOM),
            &swap_fee.recipient,
        ));
    }

    Ok(response)
}
