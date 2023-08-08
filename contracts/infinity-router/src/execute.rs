use std::iter::zip;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, NftOrder, SwapParams};
use crate::nfts_for_tokens_iterators::{NftForTokensQuote, NftForTokensSource, NftsForTokens};
use crate::state::INFINITY_GLOBAL;
use crate::tokens_for_nfts_iterators::{TokensForNftQuote, TokensForNftSource, TokensForNfts};

use cosmwasm_std::{
    coin, ensure, ensure_eq, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Uint128,
    WasmMsg,
};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use infinity_global::load_global_config;
use infinity_pair::msg::ExecuteMsg as PairExecuteMsg;
use infinity_shared::InfinityError;
use sg_marketplace_common::address::address_or;
use sg_marketplace_common::coin::transfer_coin;
use sg_marketplace_common::nft::transfer_nft;
use sg_std::Response;
use stargaze_royalty_registry::fetch_or_set_royalties;

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
            nft_orders,
            swap_params,
            filter_sources,
        } => execute_swap_nfts_for_tokens(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            denom,
            nft_orders,
            swap_params.unwrap_or_default(),
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
            swap_params.unwrap_or_default(),
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
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParams,
    filter_sources: Vec<NftForTokensSource>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let global_config = load_global_config(&deps.querier, &infinity_global)?;

    let response = Response::new();
    let (royalty_entry, mut response) = fetch_or_set_royalties(
        deps.as_ref(),
        &global_config.royalty_registry,
        &collection,
        Some(&infinity_global),
        response,
    )
    .unwrap();

    let iterator = NftsForTokens::initialize(
        deps.as_ref(),
        global_config,
        collection.clone(),
        denom.clone(),
        royalty_entry,
        filter_sources,
    );

    let requested_swaps = nft_orders.len();
    let quotes = iterator.take(requested_swaps).collect::<Vec<NftForTokensQuote>>();

    let mut num_swaps = 0u32;
    for (nft_order, quote) in zip(nft_orders, quotes) {
        response =
            transfer_nft(&collection, &nft_order.input_token_id, &env.contract.address, response);

        if quote.amount < nft_order.min_output {
            break;
        }

        match quote.source {
            NftForTokensSource::Infinity {} => {
                response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: quote.address.to_string(),
                    msg: to_binary(&PairExecuteMsg::SwapNftForTokens {
                        token_id: nft_order.input_token_id,
                        min_output: coin(nft_order.min_output.u128(), &denom),
                        asset_recipient: swap_params.asset_recipient.clone(),
                    })?,
                    funds: vec![],
                }))
            },
        }

        num_swaps += 1;
    }

    ensure!(num_swaps > 0, ContractError::SwapError("no swaps were executed".to_string()));

    if num_swaps < (requested_swaps as u32) && !swap_params.robust.unwrap_or(false) {
        return Err(ContractError::SwapError(format!(
            "unable to swap all nfts for tokens, requested swaps: {}, actual swaps: {}",
            requested_swaps, num_swaps
        )));
    }

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
    swap_params: SwapParams,
    filter_sources: Vec<TokensForNftSource>,
) -> Result<Response, ContractError> {
    let mut received_amount = must_pay(&info, &denom)?;
    let expected_amount = max_inputs.iter().sum::<Uint128>();
    ensure_eq!(
        received_amount,
        expected_amount,
        InfinityError::InsufficientFunds {
            expected: coin(expected_amount.u128(), &denom),
        }
    );

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let global_config = load_global_config(&deps.querier, &infinity_global)?;

    let response = Response::new();
    let (royalty_entry, mut response) = fetch_or_set_royalties(
        deps.as_ref(),
        &global_config.royalty_registry,
        &collection,
        Some(&infinity_global),
        response,
    )
    .unwrap();

    let iterator = TokensForNfts::initialize(
        deps.as_ref(),
        global_config,
        collection,
        denom.clone(),
        royalty_entry,
        filter_sources,
    );

    let requested_swaps = max_inputs.len();
    let quotes = iterator.take(requested_swaps).collect::<Vec<TokensForNftQuote>>();

    let mut num_swaps = 0u32;
    for (max_input, quote) in zip(max_inputs, quotes) {
        if max_input > quote.amount {
            break;
        }

        match quote.source {
            TokensForNftSource::Infinity {} => {
                response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: quote.address.to_string(),
                    msg: to_binary(&PairExecuteMsg::SwapTokensForAnyNft {
                        asset_recipient: swap_params.asset_recipient.clone(),
                    })?,
                    funds: vec![coin(quote.amount.u128(), &denom)],
                }))
            },
        }

        received_amount -= quote.amount;
        num_swaps += 1;
    }

    ensure!(num_swaps > 0, ContractError::SwapError("no swaps were executed".to_string()));

    if num_swaps < (requested_swaps as u32) && !swap_params.robust.unwrap_or(false) {
        return Err(ContractError::SwapError(format!(
            "unable to swap all tokens for nfts, requested swaps: {}, actual swaps: {}",
            requested_swaps, num_swaps
        )));
    }

    if !received_amount.is_zero() {
        let recipient =
            address_or(maybe_addr(deps.api, swap_params.asset_recipient)?.as_ref(), &info.sender);
        response = transfer_coin(coin(received_amount.u128(), &denom), &recipient, response);
    }

    Ok(response)
}
