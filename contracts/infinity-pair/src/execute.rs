use crate::error::ContractError;
use crate::helpers::{load_pair, load_payout_context, only_active, only_pair_owner};
use crate::msg::ExecuteMsg;
use crate::pair::Pair;
use crate::state::{BondingCurve, PairType, TokenId, NFT_DEPOSITS};

use cosmwasm_std::{
    coin, ensure, ensure_eq, has_coins, Addr, Coin, DepsMut, Env, MessageInfo, Order, StdResult,
    Uint128,
};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use infinity_shared::InfinityError;
use sg_marketplace_common::address::address_or;
use sg_marketplace_common::coin::transfer_coin;
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

    let payout_context =
        load_payout_context(deps.as_ref(), &pair.immutable.collection, &pair.immutable.denom)?;

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
            token_ids,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_nfts(deps, info, pair, token_ids)
        },
        ExecuteMsg::WithdrawAnyNfts {
            limit,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_any_nfts(deps, info, pair, limit)
        },
        ExecuteMsg::DepositTokens {} => {
            only_pair_owner(&info.sender, &pair)?;
            execute_deposit_tokens(deps, info, env, pair)
        },
        ExecuteMsg::WithdrawTokens {
            amount,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_tokens(deps, info, env, pair, amount)
        },
        ExecuteMsg::WithdrawAllTokens {} => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_all_tokens(deps, info, env, pair)
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
    token_ids: Vec<String>,
) -> Result<(Pair, Response), ContractError> {
    ensure!(
        !token_ids.is_empty(),
        InfinityError::InvalidInput("token_ids should not be empty".to_string())
    );

    let mut response = Response::new();

    for token_id in &token_ids {
        ensure!(
            NFT_DEPOSITS.has(deps.storage, token_id.to_string()),
            InfinityError::InvalidInput("token_id is not owned by pair".to_string())
        );
        NFT_DEPOSITS.remove(deps.storage, token_id.to_string());

        pair.internal.total_nfts -= 1u64;

        response =
            transfer_nft(&pair.immutable.collection, token_id, &pair.asset_recipient(), response);
    }

    Ok((pair, response))
}

pub fn execute_withdraw_any_nfts(
    deps: DepsMut,
    info: MessageInfo,
    pair: Pair,
    limit: u32,
) -> Result<(Pair, Response), ContractError> {
    let token_ids = NFT_DEPOSITS
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit as usize)
        .map(|item| item.map(|(v, _)| v))
        .collect::<StdResult<Vec<String>>>()?;

    execute_withdraw_nfts(deps, info, pair, token_ids)
}

pub fn execute_deposit_tokens(
    _deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    mut pair: Pair,
) -> Result<(Pair, Response), ContractError> {
    pair.total_tokens += must_pay(&info, &pair.immutable.denom)?;

    Ok((pair, Response::new()))
}

pub fn execute_withdraw_tokens(
    _deps: DepsMut,
    _info: MessageInfo,
    _env: Env,
    mut pair: Pair,
    amount: Uint128,
) -> Result<(Pair, Response), ContractError> {
    ensure!(
        amount <= pair.total_tokens,
        InfinityError::InvalidInput("amount exceeds total tokens".to_string())
    );

    pair.total_tokens -= amount;

    let mut response = Response::new();

    response = transfer_coin(
        coin(amount.u128(), pair.immutable.denom.clone()),
        &pair.asset_recipient(),
        response,
    );

    Ok((pair, response))
}

pub fn execute_withdraw_all_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    pair: Pair,
) -> Result<(Pair, Response), ContractError> {
    let total_tokens = pair.total_tokens.clone();
    execute_withdraw_tokens(deps, info, env, pair, total_tokens)
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
    let max_input = must_pay(&info, &pair.immutable.denom)?;

    let quote_summary = pair
        .internal
        .buy_from_pair_quote_summary
        .as_ref()
        .ok_or(ContractError::InvalidPair("pair cannot produce quote".to_string()))?;

    ensure!(
        max_input >= quote_summary.total(),
        ContractError::InvalidPairQuote("payment required is greater than max input".to_string())
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
    let nft_recipient = address_or(asset_recipient.as_ref(), &info.sender);
    response = transfer_nft(&pair.immutable.collection, &token_id, &nft_recipient, response);
    NFT_DEPOSITS.remove(deps.storage, token_id);

    // Refund excess tokens
    let refund_amount = max_input - quote_summary.total();
    if !refund_amount.is_zero() {
        response = transfer_coin(
            coin(refund_amount.u128(), &pair.immutable.denom),
            &info.sender,
            response,
        );
    }

    // Update pair state
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
