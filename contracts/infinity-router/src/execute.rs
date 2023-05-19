use crate::error::ContractError;
use crate::helpers::{
    build_swap_context, find_highest_sell_to_offer, find_lowest_buy_any_nft_offer,
    find_lowest_buy_specific_nft_offer, only_self_callable,
};
use crate::msg::{ExecuteMsg, NftOrder, SwapParams};
use crate::state::{INFINITY_GLOBAL, NFT_ORDERS, SWAP_CONTEXT};

use cosmwasm_std::{ensure, to_binary, DepsMut, Env, MessageInfo, SubMsg, Uint128, WasmMsg};
use cw_utils::must_pay;
use infinity_shared::global::load_global_config;
use infinity_shared::shared::only_nft_owner;
use sg_marketplace_common::transfer_nft;
use sg_std::{Response, NATIVE_DENOM};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let _api = deps.api;

    match msg {
        ExecuteMsg::SwapNftsForTokens {
            collection,
            nft_orders,
            swap_params,
        } => execute_swap_nfts_for_tokens(deps, env, info, collection, nft_orders, swap_params),
        ExecuteMsg::SwapNftsForTokensInternal {} => {
            execute_swap_nfts_for_tokens_internal(deps, env, info)
        },
        ExecuteMsg::SwapTokensForSpecificNfts {
            collection,
            nft_orders,
            swap_params,
        } => execute_swap_tokens_for_specific_nfts(
            deps,
            env,
            info,
            collection,
            nft_orders,
            swap_params,
        ),
        ExecuteMsg::SwapTokensForSpecificNftsInternal {} => {
            execute_swap_tokens_for_specific_nfts_internal(deps, env, info)
        },
        ExecuteMsg::SwapTokensForAnyNfts {
            collection,
            orders,
            swap_params,
        } => execute_swap_tokens_for_any_nfts(deps, env, info, collection, orders, swap_params),
        ExecuteMsg::SwapTokensForAnyNftsInternal {} => {
            execute_swap_tokens_for_any_nfts_internal(deps, env, info)
        },
        ExecuteMsg::CleanupSwapContext {} => execute_cleanup_swap_context(deps, env, info),
    }
}

pub fn execute_swap_nfts_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    ensure!(env.block.time <= swap_params.deadline, ContractError::DeadlinePassed);

    let swap_context = build_swap_context(deps.api, &info, collection, swap_params)?;
    SWAP_CONTEXT.save(deps.storage, &swap_context)?;

    ensure!(
        nft_orders.len() > 0,
        ContractError::InvalidInput("nft_orders should not be empty".to_string())
    );

    for nft_order in &nft_orders {
        NFT_ORDERS.push_back(deps.storage, &nft_order)?;
    }

    let mut response = Response::new();
    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapNftsForTokensInternal {})?,
        funds: vec![],
    }));
    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::CleanupSwapContext {})?,
        funds: vec![],
    }));

    Ok(response)
}

pub fn execute_swap_nfts_for_tokens_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    only_self_callable(&info, &env)?;

    let mut response = Response::new();

    let nft_order = NFT_ORDERS.pop_front(deps.storage)?;
    ensure!(
        nft_order.is_some(),
        ContractError::InternalError("nft_orders should not be empty".to_string())
    );

    let nft_order = nft_order.unwrap();

    let swap_context = SWAP_CONTEXT.load(deps.storage)?;

    only_nft_owner(
        &deps.querier,
        deps.api,
        &swap_context.original_sender,
        &swap_context.collection,
        &nft_order.token_id,
    )?;
    response = response.add_submessage(transfer_nft(
        &swap_context.collection,
        &nft_order.token_id,
        &env.contract.address,
    ));

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    let best_offer = find_highest_sell_to_offer(
        &deps.querier,
        &global_config.infinity_index,
        &global_config.marketplace,
        &swap_context.collection,
        &nft_order,
    )?;

    if let Some(best_offer) = best_offer {
        response = best_offer.place_sell_order(None, response);
    } else {
        if swap_context.robust {
            return Ok(response);
        } else {
            return Err(ContractError::UnableToMatchOrder);
        }
    }

    Ok(response)
}

pub fn execute_swap_tokens_for_specific_nfts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    ensure!(env.block.time <= swap_params.deadline, ContractError::DeadlinePassed);

    let mut swap_context = build_swap_context(deps.api, &info, collection, swap_params)?;
    SWAP_CONTEXT.save(deps.storage, &swap_context)?;

    ensure!(
        nft_orders.len() > 0,
        ContractError::InvalidInput("nft_orders should not be empty".to_string())
    );

    for nft_order in &nft_orders {
        NFT_ORDERS.push_back(deps.storage, &nft_order)?;
    }

    swap_context.balance = must_pay(&info, NATIVE_DENOM)?;
    let expected_funds = nft_orders.iter().fold(Uint128::zero(), |acc, no| acc + no.amount);
    ensure!(
        swap_context.balance >= expected_funds,
        ContractError::InsufficientFunds {
            expected: expected_funds,
            received: swap_context.balance
        }
    );

    let mut response = Response::new();
    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapTokensForSpecificNftsInternal {})?,
        funds: vec![],
    }));
    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::CleanupSwapContext {})?,
        funds: vec![],
    }));

    Ok(response)
}

pub fn execute_swap_tokens_for_specific_nfts_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    only_self_callable(&info, &env)?;

    let mut response = Response::new();

    let nft_order = NFT_ORDERS.pop_front(deps.storage)?;
    ensure!(
        nft_order.is_some(),
        ContractError::InternalError("nft_orders should not be empty".to_string())
    );

    let nft_order = nft_order.unwrap();

    let mut swap_context = SWAP_CONTEXT.load(deps.storage)?;
    ensure!(
        swap_context.balance >= nft_order.amount,
        ContractError::InsufficientFunds {
            expected: nft_order.amount,
            received: swap_context.balance
        }
    );

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    let best_offer = find_lowest_buy_specific_nft_offer(
        &deps.querier,
        &global_config.infinity_factory,
        &global_config.marketplace,
        &swap_context.collection,
        &nft_order,
    )?;

    if let Some(best_offer) = best_offer {
        response = best_offer.place_buy_order_for_specific_nft(None, response);
        swap_context.balance -= best_offer.sale_price;
        SWAP_CONTEXT.save(deps.storage, &swap_context)?;
    } else {
        if swap_context.robust {
            return Ok(response);
        } else {
            return Err(ContractError::UnableToMatchOrder);
        }
    }

    Ok(response)
}

pub fn execute_swap_tokens_for_any_nfts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
    orders: Vec<Uint128>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    ensure!(env.block.time <= swap_params.deadline, ContractError::DeadlinePassed);

    let mut swap_context = build_swap_context(deps.api, &info, collection, swap_params)?;
    SWAP_CONTEXT.save(deps.storage, &swap_context)?;

    ensure!(
        orders.len() > 0,
        ContractError::InvalidInput("orders should not be empty".to_string())
    );

    swap_context.balance = must_pay(&info, NATIVE_DENOM)?;
    let expected_funds = orders.iter().sum();
    ensure!(
        swap_context.balance >= expected_funds,
        ContractError::InsufficientFunds {
            expected: expected_funds,
            received: swap_context.balance
        }
    );

    for order in orders {
        NFT_ORDERS.push_back(
            deps.storage,
            &NftOrder {
                token_id: "".to_string(),
                amount: order,
            },
        )?;
    }

    let mut response = Response::new();
    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapTokensForAnyNftsInternal {})?,
        funds: vec![],
    }));
    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::CleanupSwapContext {})?,
        funds: vec![],
    }));

    Ok(response)
}

pub fn execute_swap_tokens_for_any_nfts_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    only_self_callable(&info, &env)?;

    let mut response = Response::new();

    let nft_order = NFT_ORDERS.pop_front(deps.storage)?;
    ensure!(
        nft_order.is_some(),
        ContractError::InternalError("nft_orders should not be empty".to_string())
    );

    let max_input = nft_order.unwrap().amount;

    let mut swap_context = SWAP_CONTEXT.load(deps.storage)?;
    ensure!(
        swap_context.balance >= max_input,
        ContractError::InsufficientFunds {
            expected: max_input,
            received: swap_context.balance
        }
    );

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    let best_offer = find_lowest_buy_any_nft_offer(
        &deps.querier,
        &global_config.infinity_factory,
        &global_config.marketplace,
        &swap_context.collection,
        max_input,
    )?;

    if let Some(best_offer) = best_offer {
        response = best_offer.place_buy_order_for_any_nft(None, response);
        swap_context.balance -= best_offer.sale_price;
        SWAP_CONTEXT.save(deps.storage, &swap_context)?;
    } else {
        if swap_context.robust {
            return Ok(response);
        } else {
            return Err(ContractError::UnableToMatchOrder);
        }
    }

    Ok(response)
}

pub fn execute_cleanup_swap_context(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    only_self_callable(&info, &env)?;

    SWAP_CONTEXT.remove(deps.storage);

    loop {
        let nft_order = NFT_ORDERS.pop_front(deps.storage)?;
        if nft_order.is_none() {
            break;
        }
    }

    Ok(Response::new())
}
