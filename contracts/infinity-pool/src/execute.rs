use crate::msg::ExecuteMsg;
use crate::pool::Pool;
use crate::state::GLOBAL_GOV;
use crate::{
    error::ContractError,
    state::{INFINITY_INDEX, POOL_CONFIG},
};

use cosmwasm_std::{
    coin, ensure, to_binary, Addr, DepsMut, Env, MessageInfo, SubMsg, Uint128, WasmMsg,
};
use cw_utils::{maybe_addr, nonpayable};
use infinity_index::msg::ExecuteMsg as InfinityIndexExecuteMsg;
use infinity_shared::shared::{load_marketplace_params, only_nft_owner};
use sg_marketplace_common::{
    calculate_nft_sale_fees, load_collection_royalties, payout_nft_sale_fees, transfer_nft,
};
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
        ExecuteMsg::SetIsActive {
            is_active,
        } => execute_set_is_active(deps, info, env, is_active),
        ExecuteMsg::SwapNftsForTokens {
            token_id,
            min_output,
            asset_recipient,
            finder,
        } => execute_swap_nfts_for_tokens(
            deps,
            info,
            env,
            token_id,
            min_output,
            api.addr_validate(&asset_recipient)?,
            maybe_addr(api, finder)?,
        ),
    }
}

/// Execute a SwapNftsForTokens message
pub fn execute_swap_nfts_for_tokens(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: String,
    min_output: Uint128,
    seller_recipient: Addr,
    finder: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = Pool::new(POOL_CONFIG.load(deps.storage)?);

    only_nft_owner(&deps.querier, deps.api, &info.sender, pool.collection(), &token_id)?;

    ensure!(pool.can_buy_nfts(), ContractError::InvalidPool("pool cannot buy nfts".to_string()));
    ensure!(pool.is_active(), ContractError::InvalidPool("pool is not active".to_string()));

    let mut response = Response::new();

    let marketplace = GLOBAL_GOV.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(&deps.querier, &marketplace)?;
    let mut total_tokens = deps.querier.query_balance(&env.contract.address, NATIVE_DENOM)?.amount;

    let sale_price = pool.get_sell_to_pool_quote(total_tokens, marketplace_params.min_price)?;

    ensure!(
        sale_price >= min_output,
        ContractError::InvalidPoolQuote("sale price is below min output".to_string())
    );

    total_tokens -= sale_price;

    let update_result = pool.update_spot_price_after_buy(total_tokens);
    let next_sell_to_pool_quote = match update_result.is_ok() {
        true => pool.get_sell_to_pool_quote(total_tokens, marketplace_params.min_price).ok(),
        false => None,
    };

    let infinity_index = INFINITY_INDEX.load(deps.storage)?;

    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: infinity_index.to_string(),
        msg: to_binary(&InfinityIndexExecuteMsg::UpdateSellToPoolQuote {
            collection: pool.collection().to_string(),
            quote_price: next_sell_to_pool_quote,
        })?,
        funds: vec![],
    }));

    // Transfer NFT to pool recipient or reinvest into pool
    let nft_recipient = match pool.reinvest_nfts() {
        true => &env.contract.address,
        false => pool.recipient(),
    };
    response = response.add_submessage(transfer_nft(pool.collection(), &token_id, nft_recipient));

    let royalty_info = load_collection_royalties(&deps.querier, deps.api, pool.collection())?;
    let tx_fees = calculate_nft_sale_fees(
        sale_price,
        marketplace_params.trading_fee_percent,
        seller_recipient,
        finder,
        None,
        royalty_info,
    )?;
    println!("tx_fees: {:?}", tx_fees);
    response = payout_nft_sale_fees(response, tx_fees, None)?;

    pool.save(deps.storage)?;

    Ok(response)
}

pub fn execute_set_is_active(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    is_active: bool,
) -> Result<Response, ContractError> {
    let mut pool = Pool::new(POOL_CONFIG.load(deps.storage)?);

    ensure!(
        &info.sender == pool.owner(),
        ContractError::Unauthorized("sender is not owner".to_string())
    );

    pool.set_is_active(is_active);
    pool.save(deps.storage)?;

    let mut response = Response::new();

    let marketplace = GLOBAL_GOV.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(&deps.querier, &marketplace)?;
    let total_tokens = deps.querier.query_balance(&env.contract.address, NATIVE_DENOM)?.amount;
    let next_sell_to_pool_quote =
        pool.get_sell_to_pool_quote(total_tokens, marketplace_params.min_price).ok();

    let infinity_index = INFINITY_INDEX.load(deps.storage)?;

    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: infinity_index.to_string(),
        msg: to_binary(&InfinityIndexExecuteMsg::UpdateSellToPoolQuote {
            collection: pool.collection().to_string(),
            quote_price: next_sell_to_pool_quote,
        })?,
        funds: vec![],
    }));

    Ok(response)
}
