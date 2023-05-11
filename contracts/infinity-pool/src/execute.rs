use crate::helpers::{load_escrow_nft_pool, load_pool};
use crate::msg::ExecuteMsg;
use crate::state::NFT_DEPOSITS;
use crate::{error::ContractError, helpers::load_escrow_token_pool};

use cosmwasm_std::{attr, coin, ensure, Addr, DepsMut, Env, Event, MessageInfo, Uint128};
use cw_utils::{maybe_addr, nonpayable};
use infinity_shared::shared::only_nft_owner;
use sg_marketplace_common::{bank_send, transfer_nft};
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
    let api = deps.api;

    match msg {
        ExecuteMsg::DepositNfts {
            collection,
            token_ids,
        } => execute_deposit_nfts(deps, info, env, api.addr_validate(&collection)?, token_ids),
        ExecuteMsg::WithdrawTokens {
            amount,
            asset_recipient,
        } => execute_withdraw_tokens(deps, info, env, amount, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::WithdrawAllTokens {
            asset_recipient,
        } => execute_withdraw_all_tokens(deps, info, env, maybe_addr(api, asset_recipient)?),
        // ExecuteMsg::SetIsActive {
        //     is_active,
        // } => execute_set_is_active(deps, info, env, is_active),
        // ExecuteMsg::SwapNftsForTokens {
        //     token_id,
        //     min_output,
        //     asset_recipient,
        //     finder,
        // } => execute_swap_nfts_for_tokens(
        //     deps,
        //     info,
        //     env,
        //     token_id,
        //     min_output,
        //     api.addr_validate(&asset_recipient)?,
        //     maybe_addr(api, finder)?,
        // ),
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

    let mut escrow_nft_pool =
        load_escrow_nft_pool(&env.contract.address, deps.storage, &deps.querier)?;

    ensure!(
        escrow_nft_pool.owner() == &info.sender,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string())
    );
    ensure!(
        escrow_nft_pool.collection() == &collection,
        ContractError::InvalidInput("invalid collection".to_string())
    );
    ensure!(
        !token_ids.is_empty(),
        ContractError::InvalidInput("token_ids should not be empty".to_string())
    );

    let mut response = Response::new();

    for token_id in &token_ids {
        only_nft_owner(
            &deps.querier,
            deps.api,
            &info.sender,
            escrow_nft_pool.collection(),
            token_id,
        )?;
        response = response.add_submessage(transfer_nft(
            escrow_nft_pool.collection(),
            &token_id,
            &env.contract.address,
        ));
        NFT_DEPOSITS.save(deps.storage, token_id.to_string(), &true)?;
    }

    escrow_nft_pool.set_total_nfts(token_ids.len() as u64);
    escrow_nft_pool.save(deps.storage)?;

    response =
        response.add_event(escrow_nft_pool.create_event("deposit-nfts", vec!["total_nfts"])?);

    Ok(response)
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
        pool.owner() == &info.sender,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string())
    );
    ensure!(
        &amount <= pool.total_tokens(),
        ContractError::InvalidInput("amount exceeds total tokens".to_string())
    );

    let total_tokens = pool.total_tokens() - amount;
    pool.set_total_tokens(total_tokens);
    pool.save(deps.storage)?;

    let mut response = Response::new();
    let recipient = asset_recipient.unwrap_or(info.sender.clone());
    response = response
        .add_submessage(bank_send(coin(amount.u128(), NATIVE_DENOM.to_string()), &recipient));

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

// /// Execute a SwapNftsForTokens message
// pub fn execute_swap_nfts_for_tokens(
//     deps: DepsMut,
//     info: MessageInfo,
//     env: Env,
//     token_id: String,
//     min_output: Uint128,
//     seller_recipient: Addr,
//     finder: Option<Addr>,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let mut pool = Pool::new(POOL_CONFIG.load(deps.storage)?);

//     only_nft_owner(&deps.querier, deps.api, &info.sender, pool.collection(), &token_id)?;

//     ensure!(pool.can_buy_nfts(), ContractError::InvalidPool("pool cannot buy nfts".to_string()));
//     ensure!(pool.is_active(), ContractError::InvalidPool("pool is not active".to_string()));

//     let mut response = Response::new();

//     let marketplace = GLOBAL_GOV.load(deps.storage)?;
//     let marketplace_params = load_marketplace_params(&deps.querier, &marketplace)?;
//     let mut total_tokens = deps.querier.query_balance(&env.contract.address, NATIVE_DENOM)?.amount;

//     let sale_price = pool.get_sell_to_pool_quote(total_tokens, marketplace_params.min_price)?;

//     ensure!(
//         sale_price >= min_output,
//         ContractError::InvalidPoolQuote("sale price is below min output".to_string())
//     );

//     total_tokens -= sale_price;

//     let update_result = pool.update_spot_price_after_buy(total_tokens);
//     let next_sell_to_pool_quote = match update_result.is_ok() {
//         true => pool.get_sell_to_pool_quote(total_tokens, marketplace_params.min_price).ok(),
//         false => None,
//     };

//     let infinity_index = INFINITY_INDEX.load(deps.storage)?;

//     response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
//         contract_addr: infinity_index.to_string(),
//         msg: to_binary(&InfinityIndexExecuteMsg::UpdateSellToPoolQuote {
//             collection: pool.collection().to_string(),
//             quote_price: next_sell_to_pool_quote,
//         })?,
//         funds: vec![],
//     }));

//     // Transfer NFT to pool recipient or reinvest into pool
//     let nft_recipient = match pool.reinvest_nfts() {
//         true => &env.contract.address,
//         false => pool.recipient(),
//     };
//     response = response.add_submessage(transfer_nft(pool.collection(), &token_id, nft_recipient));

//     let royalty_info = load_collection_royalties(&deps.querier, deps.api, pool.collection())?;
//     let tx_fees = calculate_nft_sale_fees(
//         sale_price,
//         marketplace_params.trading_fee_percent,
//         seller_recipient,
//         finder,
//         None,
//         royalty_info,
//     )?;
//     println!("tx_fees: {:?}", tx_fees);
//     response = payout_nft_sale_fees(response, tx_fees, None)?;

//     pool.save(deps.storage)?;

//     Ok(response)
// }

// pub fn execute_set_is_active(
//     deps: DepsMut,
//     info: MessageInfo,
//     env: Env,
//     is_active: bool,
// ) -> Result<Response, ContractError> {
//     let mut pool = Pool::new(POOL_CONFIG.load(deps.storage)?);

//     ensure!(
//         &info.sender == pool.owner(),
//         ContractError::Unauthorized("sender is not owner".to_string())
//     );

//     pool.set_is_active(is_active);
//     pool.save(deps.storage)?;

//     let mut response = Response::new();

//     let marketplace = GLOBAL_GOV.load(deps.storage)?;
//     let marketplace_params = load_marketplace_params(&deps.querier, &marketplace)?;
//     let total_tokens = deps.querier.query_balance(&env.contract.address, NATIVE_DENOM)?.amount;
//     let next_sell_to_pool_quote =
//         pool.get_sell_to_pool_quote(total_tokens, marketplace_params.min_price).ok();

//     let infinity_index = INFINITY_INDEX.load(deps.storage)?;

//     response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
//         contract_addr: infinity_index.to_string(),
//         msg: to_binary(&InfinityIndexExecuteMsg::UpdateSellToPoolQuote {
//             collection: pool.collection().to_string(),
//             quote_price: next_sell_to_pool_quote,
//         })?,
//         funds: vec![],
//     }));

//     Ok(response)
// }
