use crate::helpers::{load_pair, only_active, only_pair_owner};
use crate::msg::ExecuteMsg;
use crate::pair::Pair;
use crate::state::{BondingCurve, PairType, TokenId, NFT_DEPOSITS};
use crate::{error::ContractError, state::INFINITY_GLOBAL};

use cosmwasm_std::{
    coin, ensure, ensure_eq, has_coins, Addr, Coin, DepsMut, Env, MessageInfo, Order, StdResult,
    Uint128,
};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use infinity_global::load_global_config;
use infinity_global::GlobalConfig;
use infinity_shared::InfinityError;
use sg_marketplace_common::address::address_or;
use sg_marketplace_common::coin::transfer_coin;
use sg_marketplace_common::nft::{only_with_owner_approval, transfer_nft};
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use stargaze_royalty_registry::fetch_or_set_royalties;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    let pair = load_pair(&env.contract.address, deps.storage, &deps.querier)?;
    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;

    match msg {
        ExecuteMsg::ReceiveNft(cw721_receive_msg) => {
            nonpayable(&info)?;
            only_pair_owner(&api.addr_validate(&cw721_receive_msg.sender)?, &pair)?;
            execute_receive_nft(deps, info, global_config, pair, cw721_receive_msg.token_id)
        },
        ExecuteMsg::WithdrawNfts {
            token_ids,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_nfts(deps, info, global_config, pair, token_ids)
        },
        ExecuteMsg::WithdrawAnyNfts {
            limit,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_any_nfts(deps, info, global_config, pair, limit)
        },
        ExecuteMsg::DepositTokens {} => {
            only_pair_owner(&info.sender, &pair)?;
            execute_deposit_tokens(deps, info, env, global_config, pair)
        },
        ExecuteMsg::WithdrawTokens {
            amount,
        } => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_tokens(deps, info, env, global_config, pair, amount)
        },
        ExecuteMsg::WithdrawAllTokens {} => {
            nonpayable(&info)?;
            only_pair_owner(&info.sender, &pair)?;
            execute_withdraw_all_tokens(deps, info, env, global_config, pair)
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
                global_config,
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
                global_config,
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
                global_config,
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
                global_config,
                pair,
                maybe_addr(api, asset_recipient)?,
            )
        },
    }
}

/// Executes as CW721ReceiveMsg which is used to deposit NFTs into the pair
pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    global_config: GlobalConfig<Addr>,
    mut pair: Pair,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    let collection = info.sender;
    ensure_eq!(
        &collection,
        &pair.immutable.collection,
        InfinityError::InvalidInput("invalid collection".to_string())
    );

    pair.internal.total_nfts += Uint128::one();
    NFT_DEPOSITS.save(deps.storage, token_id.to_string(), &true)?;

    let mut response = Response::new();
    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    Ok(response)
}

/// Execute a Withdraw Nfts message
pub fn execute_withdraw_nfts(
    deps: DepsMut,
    _info: MessageInfo,
    global_config: GlobalConfig<Addr>,
    mut pair: Pair,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    ensure!(
        !token_ids.is_empty(),
        InfinityError::InvalidInput("token_ids should not be empty".to_string())
    );

    let mut response = Response::new();

    for token_id in &token_ids {
        if NFT_DEPOSITS.has(deps.storage, token_id.to_string()) {
            NFT_DEPOSITS.remove(deps.storage, token_id.to_string());
        } else {
            return Err(
                InfinityError::InvalidInput("token_id is not owned by pair".to_string()).into()
            );
        }
        pair.internal.total_nfts -= Uint128::one();

        response =
            transfer_nft(&pair.immutable.collection, &token_id, &pair.asset_recipient(), response);
    }

    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    Ok(response)
}

/// Execute a WithdrawAllNfts message
pub fn execute_withdraw_any_nfts(
    deps: DepsMut,
    info: MessageInfo,
    global_config: GlobalConfig<Addr>,
    pair: Pair,
    limit: u32,
) -> Result<Response, ContractError> {
    let token_ids = NFT_DEPOSITS
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit as usize)
        .map(|item| item.map(|(v, _)| v))
        .collect::<StdResult<Vec<String>>>()?;

    execute_withdraw_nfts(deps, info, global_config, pair, token_ids)
}

/// Execute a DepositTokens message
pub fn execute_deposit_tokens(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    global_config: GlobalConfig<Addr>,
    mut pair: Pair,
) -> Result<Response, ContractError> {
    pair.total_tokens += must_pay(&info, &pair.immutable.denom)?;

    let mut response = Response::new();
    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    Ok(response)
}

/// Execute a WithdrawTokens message
pub fn execute_withdraw_tokens(
    deps: DepsMut,
    _info: MessageInfo,
    _env: Env,
    global_config: GlobalConfig<Addr>,
    mut pair: Pair,
    amount: Uint128,
) -> Result<Response, ContractError> {
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

    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    Ok(response)
}

pub fn execute_withdraw_all_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    global_config: GlobalConfig<Addr>,
    pair: Pair,
) -> Result<Response, ContractError> {
    let total_tokens =
        deps.querier.query_balance(&env.contract.address, pair.immutable.denom.clone())?.amount;

    execute_withdraw_tokens(deps, info, env, global_config, pair, total_tokens)
}

pub fn execute_update_pair_config(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    global_config: GlobalConfig<Addr>,
    mut pair: Pair,
    is_active: Option<bool>,
    pair_type: Option<PairType>,
    bonding_curve: Option<BondingCurve>,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
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

    let mut response = Response::new();
    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    Ok(response)
}

/// Execute a SwapNftForTokens message
pub fn execute_swap_nft_for_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    global_config: GlobalConfig<Addr>,
    mut pair: Pair,
    token_id: String,
    min_output: Coin,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;

    let quote_summary = pair
        .internal
        .sell_to_pair_quote_summary
        .as_ref()
        .ok_or(ContractError::InvalidPair("pair cannot produce quote".to_string()))?
        .clone();

    let seller_coin = coin(quote_summary.seller_amount.u128(), &pair.immutable.denom);
    ensure!(
        has_coins(&[seller_coin], &min_output),
        ContractError::InvalidPairQuote("seller coin is less than min output".to_string())
    );

    let (royalty_entry, mut response) = fetch_or_set_royalties(
        deps.as_ref(),
        &global_config.royalty_registry,
        &pair.immutable.collection,
        Some(&infinity_global),
        Response::new(),
    )?;

    pair.swap_nft_for_tokens(
        global_config.fair_burn_fee_percent,
        royalty_entry.as_ref().map(|e| e.share),
    );

    // Payout token fees
    let seller_recipient = address_or(asset_recipient.as_ref(), &info.sender);

    response = quote_summary.payout(
        &pair.immutable.denom,
        &global_config.fair_burn,
        royalty_entry.as_ref().map(|e| &e.recipient),
        &seller_recipient,
        response,
    );

    // Handle reinvest NFTs
    let nft_recipient = if pair.reinvest_nfts() {
        NFT_DEPOSITS.save(deps.storage, token_id.clone(), &true)?;
        env.contract.address
    } else {
        pair.asset_recipient()
    };
    response = transfer_nft(&pair.immutable.collection, &token_id, &nft_recipient, response);

    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    Ok(response)
}

pub fn execute_swap_tokens_for_specific_nft(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    global_config: GlobalConfig<Addr>,
    mut pair: Pair,
    token_id: String,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;

    let max_input = must_pay(&info, &pair.immutable.denom)?;

    let quote_summary = pair
        .internal
        .buy_from_pair_quote_summary
        .as_ref()
        .ok_or(ContractError::InvalidPair("pair cannot produce quote".to_string()))?
        .clone();

    ensure!(
        max_input >= quote_summary.total(),
        ContractError::InvalidPairQuote("payment required is greater than max input".to_string())
    );

    let (royalty_entry, mut response) = fetch_or_set_royalties(
        deps.as_ref(),
        &global_config.royalty_registry,
        &pair.immutable.collection,
        Some(&infinity_global),
        Response::new(),
    )?;

    pair.swap_tokens_for_nft(
        global_config.fair_burn_fee_percent,
        royalty_entry.as_ref().map(|e| e.share),
    );

    // Payout token fees, handle reinvest tokens
    let seller_recipient = if pair.reinvest_tokens() {
        env.contract.address
    } else {
        pair.asset_recipient()
    };
    response = quote_summary.payout(
        &pair.immutable.denom,
        &global_config.fair_burn,
        royalty_entry.as_ref().map(|e| &e.recipient),
        &seller_recipient,
        response,
    );

    let nft_recipient = address_or(asset_recipient.as_ref(), &info.sender);
    response = transfer_nft(&pair.immutable.collection, &token_id, &nft_recipient, response);
    NFT_DEPOSITS.remove(deps.storage, token_id);

    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    Ok(response)
}

pub fn execute_swap_tokens_for_any_nft(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    global_config: GlobalConfig<Addr>,
    pair: Pair,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let mut results = NFT_DEPOSITS
        .range(deps.storage, None, None, Order::Ascending)
        .take(1)
        .map(|item| item.map(|(k, _)| k))
        .collect::<StdResult<Vec<String>>>()?;

    let token_id = if let Some(id) = results.pop() {
        id
    } else {
        return Err(ContractError::InvalidPair("pair does not have any NFTs".to_string()));
    };

    execute_swap_tokens_for_specific_nft(
        deps,
        info,
        env,
        global_config,
        pair,
        token_id,
        asset_recipient,
    )
}
