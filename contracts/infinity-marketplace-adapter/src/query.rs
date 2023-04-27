use crate::helpers::{match_user_submitted_nfts, validate_user_submitted_nfts, MatchedBid};
use crate::msg::QueryMsg;
use crate::state::CONFIG;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Uint128};
use infinity_shared::interface::{
    transform_swap_params, NftOrder, NftPayment, Swap, SwapParamsInternal, SwapResponse,
    TokenPayment, TransactionType,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use infinity_shared::shared::load_marketplace_params;
use sg_marketplace_common::{calculate_nft_sale_fees, load_collection_royalties};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::SimSwapNftsForTokens {
            sender,
            collection,
            nft_orders,
            swap_params,
        } => to_binary(&query_sim_swap_nfts_for_tokens(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        )?),
    }
}

pub fn query_sim_swap_nfts_for_tokens(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    validate_user_submitted_nfts(
        deps,
        &sender,
        &collection,
        &nft_orders,
        config.max_batch_size,
    )
    .map_err(|err| StdError::generic_err(err.to_string()))?;

    let matched_user_submitted_nfts =
        match_user_submitted_nfts(deps, &env.block, &config, &collection, nft_orders)
            .map_err(|err| StdError::generic_err(err.to_string()))?;

    if !swap_params.robust {
        let missing_match = matched_user_submitted_nfts
            .iter()
            .any(|m| m.matched_bid.is_none());
        if missing_match {
            return Err(StdError::generic_err(
                "all nfts not matched with a bid".to_string(),
            ));
        }
    }

    let mut swaps: Vec<Swap> = vec![];

    let marketplace_params = load_marketplace_params(&deps.querier, &config.marketplace)?;
    let royalty_info = load_collection_royalties(&deps.querier, deps.api, &collection)?;

    for matched_order in matched_user_submitted_nfts {
        if matched_order.matched_bid.is_none() {
            continue;
        }
        let matched_bid = matched_order.matched_bid.unwrap();

        let (price, bidder, finders_fee_bps): (Uint128, Addr, Option<u64>) = match matched_bid {
            MatchedBid::Bid(bid) => (bid.price, bid.bidder, bid.finders_fee_bps),
            MatchedBid::CollectionBid(collection_bid) => (
                collection_bid.price,
                collection_bid.bidder,
                collection_bid.finders_fee_bps,
            ),
        };

        let token_id = matched_order.nft_order.token_id;

        let tx_fees = calculate_nft_sale_fees(
            price,
            marketplace_params.trading_fee_percent,
            &sender,
            swap_params.finder.as_ref(),
            finders_fee_bps,
            royalty_info.as_ref(),
        )?;

        let mut token_payments: Vec<TokenPayment> = vec![];
        if let Some(finders_fee) = tx_fees.finders_fee {
            token_payments.push(TokenPayment {
                label: "finder".to_string(),
                address: finders_fee.recipient.to_string(),
                amount: finders_fee.coin.amount,
            });
        }
        if let Some(royalty_fee) = tx_fees.royalty_fee {
            token_payments.push(TokenPayment {
                label: "royalty".to_string(),
                address: royalty_fee.recipient.to_string(),
                amount: royalty_fee.coin.amount,
            });
        }
        token_payments.push(TokenPayment {
            label: "seller".to_string(),
            address: tx_fees.seller_payment.recipient.to_string(),
            amount: tx_fees.seller_payment.coin.amount,
        });

        swaps.push(Swap {
            source: config.marketplace.to_string(),
            transaction_type: TransactionType::UserSubmitsNfts,
            sale_price: price,
            network_fee: tx_fees.fair_burn_fee,
            nft_payments: vec![NftPayment {
                label: "buyer".to_string(),
                token_id,
                address: bidder.to_string(),
            }],
            token_payments,
        });
    }

    Ok(SwapResponse { swaps })
}
