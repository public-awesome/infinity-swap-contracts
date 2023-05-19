use crate::helpers::{
    calculate_nft_sale_fees, load_pool, pay_out_nft_sale_fees, save_pool_and_update_indices,
};
use crate::msg::ExecuteMsg;
use crate::query::MAX_QUERY_LIMIT;
use crate::state::NFT_DEPOSITS;
use crate::{error::ContractError, state::INFINITY_GLOBAL};

use cosmwasm_std::{
    coin, ensure, has_coins, Addr, Decimal, DepsMut, Empty, Env, MessageInfo, Uint128,
};
use cw721_base::helpers::Cw721Contract;
use cw_utils::{maybe_addr, must_pay, nonpayable};
use infinity_shared::{global::load_global_config, shared::only_nft_owner};
use sg_marketplace_common::{load_collection_royalties, transfer_coin, transfer_nft};
use sg_std::{Response, NATIVE_DENOM};
use std::marker::PhantomData;

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
        ExecuteMsg::DepositNfts {
            collection,
            token_ids,
        } => execute_deposit_nfts(deps, info, env, api.addr_validate(&collection)?, token_ids),
        ExecuteMsg::WithdrawNfts {
            token_ids,
            asset_recipient,
        } => execute_withdraw_nfts(deps, info, env, token_ids, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::WithdrawAllNfts {
            asset_recipient,
        } => execute_withdraw_all_nfts(deps, info, env, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::WithdrawTokens {
            amount,
            asset_recipient,
        } => execute_withdraw_tokens(deps, info, env, amount, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::WithdrawAllTokens {
            asset_recipient,
        } => execute_withdraw_all_tokens(deps, info, env, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::UpdatePoolConfig {
            asset_recipient,
            delta,
            spot_price,
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens,
            reinvest_nfts,
        } => execute_update_pool_config(
            deps,
            info,
            env,
            maybe_addr(api, asset_recipient)?,
            delta,
            spot_price,
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens,
            reinvest_nfts,
        ),
        ExecuteMsg::SetIsActive {
            is_active,
        } => execute_set_is_active(deps, info, env, is_active),
        ExecuteMsg::SwapNftForTokens {
            token_id,
            min_output,
            asset_recipient,
            finder,
        } => execute_swap_nft_for_tokens(
            deps,
            info,
            env,
            token_id,
            min_output,
            maybe_addr(api, asset_recipient)?,
            maybe_addr(api, finder)?,
        ),
        ExecuteMsg::SwapTokensForNft {
            token_id,
            max_input,
            asset_recipient,
            finder,
        } => execute_swap_nft_for_tokens(
            deps,
            info,
            env,
            token_id,
            max_input,
            maybe_addr(api, asset_recipient)?,
            maybe_addr(api, finder)?,
        ),
    }
}

/// Execute a DepositNfts message
pub fn execute_deposit_nfts(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    collection: Addr,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = load_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        &pool.config.owner == &info.sender,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string())
    );
    ensure!(
        &pool.config.collection == &collection,
        ContractError::InvalidInput("invalid collection".to_string())
    );
    ensure!(
        pool.can_escrow_nfts(),
        ContractError::InvalidPool("pool cannot escrow NFTs".to_string())
    );
    ensure!(
        !token_ids.is_empty(),
        ContractError::InvalidInput("token_ids should not be empty".to_string())
    );

    let mut response = Response::new();

    for token_id in &token_ids {
        only_nft_owner(&deps.querier, deps.api, &info.sender, &pool.config.collection, token_id)?;
        response = response.add_submessage(transfer_nft(
            &pool.config.collection,
            &token_id,
            &env.contract.address,
        ));
        NFT_DEPOSITS.save(deps.storage, token_id.to_string(), &true)?;
    }

    pool.config.total_nfts += token_ids.len() as u64;

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    response = save_pool_and_update_indices(
        deps.storage,
        &mut pool,
        &global_config.infinity_index,
        global_config.min_price,
        response,
    )?;

    response = response.add_event(pool.create_event("deposit-nfts", vec!["total_nfts"])?);

    Ok(response)
}

/// Execute a Withdraw Nfts message
pub fn execute_withdraw_nfts(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_ids: Vec<String>,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = load_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        &pool.config.owner == &info.sender,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string())
    );
    ensure!(
        !token_ids.is_empty(),
        ContractError::InvalidInput("token_ids should not be empty".to_string())
    );

    let mut response = Response::new();
    let recipient = asset_recipient.unwrap_or(info.sender.clone());

    for token_id in &token_ids {
        only_nft_owner(
            &deps.querier,
            deps.api,
            &env.contract.address,
            &pool.config.collection,
            &token_id,
        )?;

        response =
            response.add_submessage(transfer_nft(&pool.config.collection, &token_id, &recipient));

        if NFT_DEPOSITS.has(deps.storage, token_id.to_string()) {
            pool.config.total_nfts -= 1;
            NFT_DEPOSITS.remove(deps.storage, token_id.to_string());
        }
    }

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    response = save_pool_and_update_indices(
        deps.storage,
        &mut pool,
        &global_config.infinity_index,
        global_config.min_price,
        response,
    )?;

    response = response.add_event(pool.create_event("withdraw-nfts", vec!["total_nfts"])?);

    Ok(response)
}

/// Execute a WithdrawAllNfts message
pub fn execute_withdraw_all_nfts(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let collection =
        load_pool(&env.contract.address, deps.storage, &deps.querier)?.config.collection;

    let token_ids = Cw721Contract::<Empty, Empty>(collection, PhantomData, PhantomData)
        .tokens(&deps.querier, &env.contract.address, None, Some(MAX_QUERY_LIMIT))?
        .tokens;

    execute_withdraw_nfts(deps, info, env, token_ids, asset_recipient)
}

/// Execute a WithdrawTokens message
pub fn execute_withdraw_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = load_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        &pool.config.owner == &info.sender,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string())
    );
    ensure!(
        &amount <= &pool.total_tokens,
        ContractError::InvalidInput("amount exceeds total tokens".to_string())
    );

    pool.total_tokens -= amount;

    let mut response = Response::new();

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    response = save_pool_and_update_indices(
        deps.storage,
        &mut pool,
        &global_config.infinity_index,
        global_config.min_price,
        response,
    )?;

    let recipient = asset_recipient.unwrap_or(info.sender.clone());
    response = response
        .add_submessage(transfer_coin(coin(amount.u128(), NATIVE_DENOM.to_string()), &recipient));

    response = response.add_event(pool.create_event("withdraw-tokens", vec!["total_tokens"])?);

    Ok(response)
}

/// Execute a WithdrawAllTokens message
pub fn execute_withdraw_all_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let total_tokens = deps.querier.query_balance(&env.contract.address, NATIVE_DENOM)?.amount;
    execute_withdraw_tokens(deps, info, env, total_tokens, asset_recipient)
}

/// Execute an UpdatePoolConfig message
/// Option paramaters that are not specified will not be updated
pub fn execute_update_pool_config(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    asset_recipient: Option<Addr>,
    delta: Option<Uint128>,
    spot_price: Option<Uint128>,
    finders_fee_bps: Option<u64>,
    swap_fee_bps: Option<u64>,
    reinvest_tokens: Option<bool>,
    reinvest_nfts: Option<bool>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = load_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        &pool.config.owner == &info.sender,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string())
    );

    let mut attr_keys: Vec<&str> = vec![];
    if let Some(asset_recipient) = asset_recipient {
        pool.config.asset_recipient = Some(asset_recipient);
        attr_keys.push("asset_recipient");
    }
    if let Some(spot_price) = spot_price {
        pool.config.spot_price = spot_price;
        attr_keys.push("spot_price");
    }
    if let Some(delta) = delta {
        pool.config.delta = delta;
        attr_keys.push("delta");
    }
    if let Some(swap_fee_bps) = swap_fee_bps {
        pool.config.swap_fee_percent = Decimal::percent(swap_fee_bps);
        attr_keys.push("swap_fee_percent");
    }
    if let Some(finders_fee_bps) = finders_fee_bps {
        pool.config.finders_fee_percent = Decimal::percent(finders_fee_bps);
        attr_keys.push("finders_fee_percent");
    }
    if let Some(reinvest_tokens) = reinvest_tokens {
        pool.config.reinvest_tokens = reinvest_tokens;
        attr_keys.push("reinvest_tokens");
    }
    if let Some(reinvest_nfts) = reinvest_nfts {
        pool.config.reinvest_nfts = reinvest_nfts;
        attr_keys.push("reinvest_nfts");
    }

    let mut response = Response::new();

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    response = save_pool_and_update_indices(
        deps.storage,
        &mut pool,
        &global_config.infinity_index,
        global_config.min_price,
        response,
    )?;

    let event = pool.create_event("update-pool-config", attr_keys)?;
    response = response.add_event(event);

    Ok(response)
}

/// Execute a SetIsActive message
pub fn execute_set_is_active(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    is_active: bool,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = load_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        &pool.config.owner == &info.sender,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string())
    );

    pool.config.is_active = is_active;

    let mut response = Response::new();

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    response = save_pool_and_update_indices(
        deps.storage,
        &mut pool,
        &global_config.infinity_index,
        global_config.min_price,
        response,
    )?;

    response = response.add_event(pool.create_event("set-is-active", vec!["is_active"])?);

    Ok(response)
}

/// Execute a SwapNftForTokens message
pub fn execute_swap_nft_for_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: String,
    min_output: Uint128,
    asset_recipient: Option<Addr>,
    finder: Option<Addr>,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // ----------------------------
    // Validate inputs
    // ----------------------------
    nonpayable(&info)?;

    let mut pool = load_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        pool.can_escrow_tokens(),
        ContractError::InvalidPool("pool cannot escrow tokens".to_string())
    );

    only_nft_owner(&deps.querier, deps.api, &info.sender, &pool.config.collection, &token_id)?;

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    let sale_price = pool.get_sell_to_pool_quote(global_config.min_price)?;

    ensure!(
        sale_price >= min_output,
        ContractError::InvalidPoolQuote("sale price is below min output".to_string())
    );

    // ----------------------------
    // Write Response Messages
    // ----------------------------
    let royalty_info = load_collection_royalties(&deps.querier, deps.api, &pool.config.collection)?;
    let seller_recipient = asset_recipient.unwrap_or(info.sender.clone());
    let tx_fees = calculate_nft_sale_fees(
        sale_price,
        global_config.trading_fee_percent,
        seller_recipient,
        finder,
        pool.config.finders_fee_percent,
        royalty_info,
        pool.swap_fee_percent(),
        Some(pool.recipient().clone()),
    )?;

    // Pay the seller
    ensure!(tx_fees.seller_payment.amount > Uint128::zero(), ContractError::ZeroSellerPayment);
    response = response.add_submessage(transfer_coin(
        coin(tx_fees.seller_payment.amount.u128(), NATIVE_DENOM),
        &tx_fees.seller_payment.recipient,
    ));

    // Pay out fees
    response = pay_out_nft_sale_fees(response, tx_fees, None)?;

    // Transfer NFT to pool or pool recipient address
    let nft_recipient = if pool.should_reinvest_nfts() {
        &env.contract.address
    } else {
        pool.recipient()
    };
    response =
        response.add_submessage(transfer_nft(&pool.config.collection, &token_id, nft_recipient));

    // ----------------------------
    // Update Pool State
    // ----------------------------
    pool.total_tokens -= sale_price;

    if pool.should_reinvest_nfts() {
        pool.config.total_nfts += 1;
    }

    // Try and update spot price
    let update_spot_price_result = pool.next_spot_price_after_sell_to_pool();
    match update_spot_price_result {
        Ok(spot_price) => {
            pool.config.is_active = true;
            pool.config.spot_price = spot_price;
        },
        Err(_) => {
            pool.config.is_active = false;
        },
    };

    // ----------------------------
    // Save Pool
    // ----------------------------
    response = save_pool_and_update_indices(
        deps.storage,
        &mut pool,
        &global_config.infinity_index,
        global_config.min_price,
        response,
    )?;

    Ok(response)
}

/// Execute a SwapTokensForNft message
pub fn execute_swap_tokens_for_nft(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: String,
    max_input: Uint128,
    asset_recipient: Option<Addr>,
    finder: Option<Addr>,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // ----------------------------
    // Validate inputs
    // ----------------------------
    let received_amount = must_pay(&info, NATIVE_DENOM)?;
    ensure!(
        has_coins(&info.funds, &coin(max_input.u128(), NATIVE_DENOM)),
        ContractError::InsufficientFunds {}
    );

    let mut pool = load_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        pool.can_escrow_nfts(),
        ContractError::InvalidPool("pool cannot escrow NFTs".to_string())
    );

    ensure!(
        NFT_DEPOSITS.has(deps.storage, token_id.to_string()),
        ContractError::InvalidInput("pool does not own specified token_id".to_string())
    );

    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;
    let sale_price = pool.get_buy_from_pool_quote(global_config.min_price)?;

    ensure!(
        sale_price <= max_input,
        ContractError::InvalidPoolQuote("sale price is above max input".to_string())
    );

    // ----------------------------
    // Write Response Messages
    // ----------------------------
    let royalty_info = load_collection_royalties(&deps.querier, deps.api, &pool.config.collection)?;
    let tx_fees = calculate_nft_sale_fees(
        sale_price,
        global_config.trading_fee_percent,
        pool.recipient().clone(),
        finder,
        pool.config.finders_fee_percent,
        royalty_info,
        pool.swap_fee_percent(),
        Some(pool.recipient().clone()),
    )?;

    // If reinvest_tokens is true, do nothing as pool already owns the tokens
    // If reinvest_tokens is false, transfer tokens to pool recipient
    let seller_amount = tx_fees.seller_payment.amount;
    ensure!(seller_amount > Uint128::zero(), ContractError::ZeroSellerPayment);
    if !pool.should_reinvest_tokens() {
        response = response.add_submessage(transfer_coin(
            coin(seller_amount.u128(), NATIVE_DENOM),
            &tx_fees.seller_payment.recipient,
        ));
    }

    // Pay out fees
    response = pay_out_nft_sale_fees(response, tx_fees, None)?;

    // Refund any excess tokens to the sender
    let excess_tokens = received_amount - sale_price;
    if excess_tokens > Uint128::zero() {
        response = response
            .add_submessage(transfer_coin(coin(excess_tokens.u128(), NATIVE_DENOM), &info.sender));
    }

    // Transfer NFT to the buyer
    let nft_recipient = asset_recipient.unwrap_or(info.sender.clone());
    response =
        response.add_submessage(transfer_nft(&pool.config.collection, &token_id, &nft_recipient));
    NFT_DEPOSITS.remove(deps.storage, token_id.to_string());

    // ----------------------------
    // Update Pool State
    // ----------------------------
    pool.config.total_nfts -= 1;

    if pool.should_reinvest_tokens() {
        pool.total_tokens += seller_amount;
    }

    // Try and update spot price
    let update_spot_price_result = pool.next_spot_price_after_buy_from_pool();
    match update_spot_price_result {
        Ok(spot_price) => {
            pool.config.is_active = true;
            pool.config.spot_price = spot_price;
        },
        Err(_) => {
            pool.config.is_active = false;
        },
    };

    // ----------------------------
    // Save Pool
    // ----------------------------
    response = save_pool_and_update_indices(
        deps.storage,
        &mut pool,
        &global_config.infinity_index,
        global_config.min_price,
        response,
    )?;

    Ok(response)
}
