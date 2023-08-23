use crate::error::ContractError;
use crate::helpers::{load_pair, load_payout_context, only_active, only_pair_owner};
use crate::msg::ExecuteMsg;
use crate::pair::Pair;
use crate::state::{BondingCurve, PairType, TokenId, INFINITY_GLOBAL, NFT_DEPOSITS};

use cosmwasm_std::{
    coin, ensure, ensure_eq, has_coins, Addr, Coin, DepsMut, Env, MessageInfo, Order, StdResult,
};
use cw721::{Cw721QueryMsg, TokensResponse};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use infinity_shared::InfinityError;
use sg_marketplace_common::address::address_or;
use sg_marketplace_common::coin::transfer_coins;
use sg_marketplace_common::nft::{only_with_owner_approval, transfer_nft};
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let pair = load_pair(&env.contract.address, deps.storage, &deps.querier)?;

    let (mut pair, mut response) = handle_execute_msg(deps.branch(), env, info, msg, pair)?;

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let payout_context = load_payout_context(
        deps.as_ref(),
        &infinity_global,
        &pair.immutable.collection,
        &pair.immutable.denom,
    )?;

    response = pair.save_and_update_indices(deps.storage, &payout_context, response)?;

    Ok(response)
}

pub fn handle_execute_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
    pair: Pair,
) -> Result<(Pair, Response), ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::ReceiveNft(cw721_receive_msg) => {
            nonpayable(&info)?;
            only_pair_owner(&api.addr_validate(&cw721_receive_msg.sender)?, &pair)?;
            execute_receive_nft(deps, info, pair, cw721_receive_msg.token_id)
        },
        ExecuteMsg::WithdrawNfts {
            collection,
            token_ids,
            asset_recipient,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_nfts(
                deps,
                info,
                pair,
                api.addr_validate(&collection)?,
                token_ids,
                maybe_addr(api, asset_recipient)?,
            )
        },
        ExecuteMsg::WithdrawAnyNfts {
            collection,
            limit,
            asset_recipient,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_any_nfts(
                deps,
                env,
                info,
                pair,
                api.addr_validate(&collection)?,
                limit,
                maybe_addr(api, asset_recipient)?,
            )
        },
        ExecuteMsg::DepositTokens {} => {
            only_pair_owner(&info.sender, &pair)?;
            execute_deposit_tokens(deps, info, env, pair)
        },
        ExecuteMsg::WithdrawTokens {
            funds,
            asset_recipient,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_tokens(deps, info, env, pair, funds, maybe_addr(api, asset_recipient)?)
        },
        ExecuteMsg::WithdrawAllTokens {
            asset_recipient,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_all_tokens(deps, info, env, pair, maybe_addr(api, asset_recipient)?)
        },
        ExecuteMsg::UpdatePairConfig {
            is_active,
            pair_type,
            bonding_curve,
            asset_recipient,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_update_pair_config(
                deps,
                info,
                env,
                pair,
                is_active,
                pair_type,
                bonding_curve,
                maybe_addr(api, asset_recipient)?,
            )
        },
        ExecuteMsg::SwapNftForTokens {
            token_id,
            min_output,
            asset_recipient,
        } => {
            nonpayable(&info)?;
            only_active(&pair)?;
            only_with_owner_approval(
                &deps.querier,
                &info,
                &pair.immutable.collection,
                &token_id,
                &env.contract.address,
            )?;
            execute_swap_nft_for_tokens(
                deps,
                info,
                env,
                pair,
                token_id,
                min_output,
                maybe_addr(api, asset_recipient)?,
            )
        },
        ExecuteMsg::SwapTokensForSpecificNft {
            token_id,
            asset_recipient,
        } => {
            only_active(&pair)?;
            execute_swap_tokens_for_specific_nft(
                deps,
                info,
                env,
                pair,
                token_id,
                maybe_addr(api, asset_recipient)?,
            )
        },
        ExecuteMsg::SwapTokensForAnyNft {
            asset_recipient,
        } => {
            only_active(&pair)?;
            execute_swap_tokens_for_any_nft(
                deps,
                info,
                env,
                pair,
                maybe_addr(api, asset_recipient)?,
            )
        },
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    mut pair: Pair,
    token_id: TokenId,
) -> Result<(Pair, Response), ContractError> {
    let collection = info.sender;
    ensure_eq!(
        &collection,
        &pair.immutable.collection,
        InfinityError::InvalidInput("invalid collection".to_string())
    );

    pair.internal.total_nfts += 1u64;
    NFT_DEPOSITS.save(deps.storage, token_id, &true)?;

    Ok((pair, Response::new()))
}

pub fn execute_withdraw_nfts(
    deps: DepsMut,
    _info: MessageInfo,
    mut pair: Pair,
    collection: Addr,
    token_ids: Vec<String>,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    ensure!(
        !token_ids.is_empty(),
        InfinityError::InvalidInput("token_ids should not be empty".to_string())
    );

    let mut response = Response::new();

    let asset_recipient = address_or(asset_recipient.as_ref(), &pair.asset_recipient());

    for token_id in &token_ids {
        response = transfer_nft(&collection, token_id, &asset_recipient, response);

        if pair.immutable.collection == collection
            && NFT_DEPOSITS.has(deps.storage, token_id.to_string())
        {
            pair.internal.total_nfts -= 1u64;
            NFT_DEPOSITS.remove(deps.storage, token_id.to_string());
        }
    }

    Ok((pair, response))
}

pub fn execute_withdraw_any_nfts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pair: Pair,
    collection: Addr,
    limit: u32,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    let token_ids = deps
        .querier
        .query_wasm_smart::<TokensResponse>(
            &collection,
            &Cw721QueryMsg::Tokens {
                owner: env.contract.address.to_string(),
                start_after: None,
                limit: Some(limit),
            },
        )?
        .tokens;

    execute_withdraw_nfts(deps, info, pair, collection, token_ids, asset_recipient)
}

pub fn execute_deposit_tokens(
    _deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    pair: Pair,
) -> Result<(Pair, Response), ContractError> {
    must_pay(&info, &pair.immutable.denom)?;
    Ok((pair, Response::new()))
}

pub fn execute_withdraw_tokens(
    _deps: DepsMut,
    _info: MessageInfo,
    _env: Env,
    mut pair: Pair,
    funds: Vec<Coin>,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    for fund in &funds {
        if fund.denom == pair.immutable.denom {
            pair.total_tokens -= fund.amount;
        }
    }

    let mut response = Response::new();

    let asset_recipient = address_or(asset_recipient.as_ref(), &pair.asset_recipient());

    response = transfer_coins(funds, &asset_recipient, response);

    Ok((pair, response))
}

pub fn execute_withdraw_all_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    pair: Pair,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    let all_tokens = deps.querier.query_all_balances(&env.contract.address)?;
    execute_withdraw_tokens(deps, info, env, pair, all_tokens, asset_recipient)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_update_pair_config(
    _deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    mut pair: Pair,
    is_active: Option<bool>,
    pair_type: Option<PairType>,
    bonding_curve: Option<BondingCurve>,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    nonpayable(&info)?;

    if let Some(is_active) = is_active {
        pair.config.is_active = is_active;
    }

    if let Some(pair_type) = pair_type {
        pair.config.pair_type = pair_type;
    }

    if let Some(bonding_curve) = bonding_curve {
        pair.config.bonding_curve = bonding_curve;
    }

    if let Some(asset_recipient) = asset_recipient {
        pair.config.asset_recipient = Some(asset_recipient);
    }

    Ok((pair, Response::new()))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_swap_nft_for_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    mut pair: Pair,
    token_id: String,
    min_output: Coin,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    let quote_summary = pair
        .internal
        .sell_to_pair_quote_summary
        .as_ref()
        .ok_or(ContractError::InvalidPair("pair cannot produce quote".to_string()))?;

    let seller_coin = coin(quote_summary.seller_amount.u128(), &pair.immutable.denom);
    ensure!(
        has_coins(&[seller_coin], &min_output),
        ContractError::InvalidPairQuote("seller coin is less than min output".to_string())
    );

    let mut response = Response::new();

    // Payout token fees
    let seller_recipient = address_or(asset_recipient.as_ref(), &info.sender);
    response = quote_summary.payout(&pair.immutable.denom, &seller_recipient, response)?;

    // Payout NFT, handle reinvest NFTs
    let nft_recipient = if pair.reinvest_nfts() {
        NFT_DEPOSITS.save(deps.storage, token_id.clone(), &true)?;
        env.contract.address
    } else {
        pair.asset_recipient()
    };
    response = transfer_nft(&pair.immutable.collection, &token_id, &nft_recipient, response);

    // Update pair state
    pair.swap_nft_for_tokens();

    Ok((pair, response))
}

pub fn execute_swap_tokens_for_specific_nft(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    mut pair: Pair,
    token_id: String,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    let received_amount = must_pay(&info, &pair.immutable.denom)?;

    let quote_summary = pair
        .internal
        .buy_from_pair_quote_summary
        .as_ref()
        .ok_or(ContractError::InvalidPair("pair cannot produce quote".to_string()))?;

    ensure_eq!(
        received_amount,
        quote_summary.total(),
        InfinityError::InvalidInput("received funds does not equal quote".to_string())
    );

    let mut response = Response::new();

    // Payout token fees, handle reinvest tokens
    let seller_recipient = if pair.reinvest_tokens() {
        env.contract.address
    } else {
        pair.asset_recipient()
    };
    response = quote_summary.payout(&pair.immutable.denom, &seller_recipient, response)?;

    // Payout NFT
    ensure!(
        NFT_DEPOSITS.has(deps.storage, token_id.clone()),
        InfinityError::InvalidInput("pair does not own NFT".to_string())
    );
    NFT_DEPOSITS.remove(deps.storage, token_id.clone());

    let nft_recipient = address_or(asset_recipient.as_ref(), &info.sender);
    response = transfer_nft(&pair.immutable.collection, &token_id, &nft_recipient, response);

    // Update pair state
    pair.total_tokens -= received_amount;
    pair.swap_tokens_for_nft();

    Ok((pair, response))
}

pub fn execute_swap_tokens_for_any_nft(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    pair: Pair,
    asset_recipient: Option<Addr>,
) -> Result<(Pair, Response), ContractError> {
    let token_id = NFT_DEPOSITS
        .range(deps.storage, None, None, Order::Ascending)
        .take(1)
        .map(|item| item.map(|(k, _)| k))
        .collect::<StdResult<Vec<String>>>()?
        .pop()
        .ok_or(ContractError::InvalidPair("pair does not have any NFTs".to_string()))?;

    execute_swap_tokens_for_specific_nft(deps, info, env, pair, token_id, asset_recipient)
}
