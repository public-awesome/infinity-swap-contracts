use crate::error::ContractError;
use crate::helpers::approve_nft;
use crate::msg::{ExecuteMsg, SellOrder, SwapParams};
use crate::nfts_for_tokens_iterators::{
    iter::NftsForTokens,
    types::{NftForTokensQuote, NftForTokensSource},
};
use crate::state::INFINITY_GLOBAL;
use crate::tokens_for_nfts_iterators::{
    types::TokensForNftQuote,
    {iter::TokensForNfts, types::TokensForNftSource},
};

use cosmwasm_std::{
    attr, coin, ensure, ensure_eq, to_binary, Addr, CosmosMsg, DepsMut, Env, Event, MessageInfo,
    Uint128, WasmMsg,
};
use cw_utils::{must_pay, nonpayable};
use infinity_pair::msg::ExecuteMsg as PairExecuteMsg;
use infinity_shared::{only_nft_owner, InfinityError};
use sg_marketplace_common::address::address_or;
use sg_marketplace_common::coin::transfer_coin;
use sg_marketplace_common::nft::transfer_nft;
use sg_std::Response;
use std::iter::zip;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::SwapNftsForTokens {
            collection,
            denom,
            sell_orders,
            swap_params,
            filter_sources,
        } => execute_swap_nfts_for_tokens(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            denom,
            sell_orders,
            swap_params.unwrap_or_default().str_to_addr(api)?,
            filter_sources.unwrap_or_default(),
        ),
        ExecuteMsg::SwapTokensForNfts {
            collection,
            denom,
            max_inputs,
            swap_params,
            filter_sources,
        } => execute_swap_tokens_for_nfts(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            denom,
            max_inputs,
            swap_params.unwrap_or_default().str_to_addr(api)?,
            filter_sources.unwrap_or_default(),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_swap_nfts_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    denom: String,
    sell_orders: Vec<SellOrder>,
    swap_params: SwapParams<Addr>,
    filter_sources: Vec<NftForTokensSource>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let iterator = NftsForTokens::initialize(
        deps.as_ref(),
        &infinity_global,
        &collection,
        &denom,
        filter_sources,
    )?;

    let requested_swaps = sell_orders.len();
    let quotes = iterator.take(requested_swaps).collect::<Vec<NftForTokensQuote>>();

    let mut response = Response::new();

    let mut num_swaps = 0u32;
    let mut volume = Uint128::zero();
    for (sell_order, quote) in zip(sell_orders, quotes) {
        if quote.amount < sell_order.min_output {
            break;
        }

        only_nft_owner(&deps.querier, &info, &collection, &sell_order.input_token_id)?;
        response =
            transfer_nft(&collection, &sell_order.input_token_id, &env.contract.address, response);

        match quote.source {
            NftForTokensSource::Infinity => {
                response =
                    approve_nft(&collection, &quote.address, &sell_order.input_token_id, response);
                response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: quote.address.to_string(),
                    msg: to_binary(&PairExecuteMsg::SwapNftForTokens {
                        token_id: sell_order.input_token_id,
                        min_output: coin(sell_order.min_output.u128(), &denom),
                        asset_recipient: Some(
                            address_or(swap_params.asset_recipient.as_ref(), &info.sender)
                                .to_string(),
                        ),
                    })?,
                    funds: vec![],
                }))
            },
        }

        num_swaps += 1;
        volume += quote.amount;
    }

    ensure!(num_swaps > 0, ContractError::SwapError("no swaps were executed".to_string()));

    if num_swaps < (requested_swaps as u32) && !swap_params.robust.unwrap_or(false) {
        return Err(ContractError::SwapError(format!(
            "unable to swap all nfts for tokens, requested swaps: {}, actual swaps: {}",
            requested_swaps, num_swaps
        )));
    }

    response = response.add_event(Event::new("router-swap-nfts-for-tokens").add_attributes(vec![
        attr("collection", collection),
        attr("denom", denom),
        attr("sender", info.sender),
        attr("num_swaps", num_swaps.to_string()),
        attr("volume", volume),
    ]));

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_swap_tokens_for_nfts(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: Addr,
    denom: String,
    max_inputs: Vec<Uint128>,
    swap_params: SwapParams<Addr>,
    filter_sources: Vec<TokensForNftSource>,
) -> Result<Response, ContractError> {
    let received_amount = must_pay(&info, &denom)?;
    let expected_amount = max_inputs.iter().sum::<Uint128>();
    ensure_eq!(
        received_amount,
        expected_amount,
        InfinityError::InsufficientFunds {
            expected: coin(expected_amount.u128(), &denom),
        }
    );

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let iterator = TokensForNfts::initialize(
        deps.as_ref(),
        &infinity_global,
        &collection,
        &denom,
        filter_sources,
    );

    let requested_swaps = max_inputs.len();
    let quotes = iterator.take(requested_swaps).collect::<Vec<TokensForNftQuote>>();

    let mut response = Response::new();

    let asset_recipient = address_or(swap_params.asset_recipient.as_ref(), &info.sender);

    let mut num_swaps = 0u32;
    let mut paid_amount = Uint128::zero();
    for (max_input, quote) in zip(max_inputs, quotes) {
        if max_input < quote.amount {
            break;
        }

        match quote.source {
            TokensForNftSource::Infinity => {
                response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: quote.address.to_string(),
                    msg: to_binary(&PairExecuteMsg::SwapTokensForAnyNft {
                        asset_recipient: Some(asset_recipient.to_string()),
                    })?,
                    funds: vec![coin(quote.amount.u128(), &denom)],
                }))
            },
        }

        paid_amount += quote.amount;
        num_swaps += 1;
    }

    ensure!(num_swaps > 0, ContractError::SwapError("no swaps were executed".to_string()));

    if num_swaps < (requested_swaps as u32) && !swap_params.robust.unwrap_or(false) {
        return Err(ContractError::SwapError(format!(
            "unable to swap all tokens for nfts, requested swaps: {}, actual swaps: {}",
            requested_swaps, num_swaps
        )));
    }

    let refund_amount = received_amount.checked_sub(paid_amount).unwrap();
    if !refund_amount.is_zero() {
        response = transfer_coin(coin(refund_amount.u128(), &denom), &asset_recipient, response);
    }

    response = response.add_event(Event::new("router-swap-tokens-for-nfts").add_attributes(vec![
        attr("collection", collection),
        attr("denom", denom),
        attr("sender", info.sender),
        attr("num_swaps", num_swaps.to_string()),
        attr("volume", paid_amount), // volume is the amount of tokens paid
    ]));

    Ok(response)
}
