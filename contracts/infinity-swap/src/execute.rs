use crate::error::ContractError;
use crate::helpers::{get_next_pair_id, only_pair_owner, store_nft_deposit};
use crate::msg::{ExecuteMsg, NftSwap, PairNftSwap, PairOptions, SwapParams};
use crate::state::{pairs, BondingCurve, Pair, PairType, SUDO_PARAMS};
// use crate::helpers::{
//     get_next_pair_counter, get_transaction_events, load_marketplace_params, only_owner,
//     prep_for_swap, remove_nft_deposit, remove_pair, save_pair, save_pairs, store_nft_deposit,
//     transfer_nft, transfer_token, update_nft_deposits, validate_nft_swaps_for_buy,
//     validate_nft_swaps_for_sell,
// };
// use crate::query::query_pair_nft_token_ids;
// use crate::swap_processor::{Swap, SwapProcessor};

use cosmwasm_std::{
    attr, coin, ensure, has_coins, Addr, Decimal, DepsMut, Env, Event, MessageInfo, Uint128,
};
use cw_utils::{maybe_addr, must_pay, nonpayable, one_coin};
use sg1::fair_burn;
use sg_marketplace_common::coin::transfer_coin;
use sg_marketplace_common::nft::transfer_nft;
use sg_std::{Response, NATIVE_DENOM};
use stargaze_fair_burn::append_fair_burn_msg;

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
        ExecuteMsg::CreatePair {
            collection,
            denom,
            pair_type,
            bonding_curve,
            pair_options,
        } => execute_create_pair(
            deps,
            info,
            api.addr_validate(&collection)?,
            denom,
            pair_type,
            bonding_curve,
            pair_options.unwrap_or_default(),
        ),
        ExecuteMsg::DepositTokens { pair_id } => execute_deposit_tokens(deps, info, pair_id),
        ExecuteMsg::DepositNfts {
            pair_id,
            collection,
            nft_token_ids,
        } => execute_deposit_nfts(
            deps,
            info,
            env,
            pair_id,
            api.addr_validate(&collection)?,
            nft_token_ids,
        ),
        ExecuteMsg::WithdrawTokens { pair_id, amount } => {
            execute_withdraw_tokens(deps, info, pair_id, amount)
        }
        ExecuteMsg::WithdrawAllTokens {
            pair_id,
            asset_recipient,
        } => execute_withdraw_all_tokens(deps, info, pair_id, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::WithdrawNfts {
            pair_id,
            nft_token_ids,
            asset_recipient,
        } => execute_withdraw_nfts(
            deps,
            info,
            pair_id,
            nft_token_ids,
            maybe_addr(api, asset_recipient)?,
        ),
        ExecuteMsg::WithdrawAllNfts {
            pair_id,
            asset_recipient,
        } => execute_withdraw_all_nfts(deps, info, pair_id, maybe_addr(api, asset_recipient)?),
        // ExecuteMsg::UpdatePairConfig {
        //     pair_id,
        //     asset_recipient,
        //     delta,
        //     spot_price,
        //     finders_fee_bps,
        //     swap_fee_bps,
        //     reinvest_tokens,
        //     reinvest_nfts,
        // } => execute_update_pair_config(
        //     deps,
        //     info,
        //     pair_id,
        //     maybe_addr(api, asset_recipient)?,
        //     delta,
        //     spot_price,
        //     finders_fee_bps,
        //     swap_fee_bps,
        //     reinvest_tokens,
        //     reinvest_nfts,
        // ),
        // ExecuteMsg::SetActivePair { pair_id, is_active } => {
        //     execute_set_active_pair(deps, info, pair_id, is_active)
        // }
        // ExecuteMsg::RemovePair {
        //     pair_id,
        //     asset_recipient,
        // } => execute_remove_pair(deps, info, pair_id, maybe_addr(api, asset_recipient)?),
        // ExecuteMsg::DirectSwapNftsForTokens {
        //     pair_id,
        //     nfts_to_swap,
        //     swap_params,
        // } => {
        //     execute_direct_swap_nfts_for_tokens(deps, env, info, pair_id, nfts_to_swap, swap_params)
        // }
        // ExecuteMsg::SwapNftsForTokens {
        //     collection,
        //     nfts_to_swap,
        //     swap_params,
        // } => execute_swap_nfts_for_tokens(
        //     deps,
        //     env,
        //     info,
        //     api.addr_validate(&collection)?,
        //     nfts_to_swap,
        //     swap_params,
        // ),
        // ExecuteMsg::DirectSwapTokensForSpecificNfts {
        //     pair_id,
        //     nfts_to_swap_for,
        //     swap_params,
        // } => execute_direct_swap_tokens_for_specific_nfts(
        //     deps,
        //     env,
        //     info,
        //     pair_id,
        //     nfts_to_swap_for,
        //     swap_params,
        // ),
        // ExecuteMsg::SwapTokensForSpecificNfts {
        //     collection,
        //     pair_nfts_to_swap_for,
        //     swap_params,
        // } => execute_swap_tokens_for_specific_nfts(
        //     deps,
        //     env,
        //     info,
        //     api.addr_validate(&collection)?,
        //     pair_nfts_to_swap_for,
        //     swap_params,
        // ),
        // ExecuteMsg::SwapTokensForAnyNfts {
        //     collection,
        //     max_expected_token_input,
        //     swap_params,
        // } => execute_swap_tokens_for_any_nfts(
        //     deps,
        //     env,
        //     info,
        //     api.addr_validate(&collection)?,
        //     max_expected_token_input,
        //     swap_params,
        // ),
    }
}

/// Execute a CreatePair message
pub fn execute_create_pair(
    deps: DepsMut,
    info: MessageInfo,
    collection: Addr,
    denom: String,
    pair_type: PairType,
    bonding_curve: BondingCurve,
    pair_options: PairOptions<String>,
) -> Result<Response, ContractError> {
    let sudo_params = SUDO_PARAMS.load(deps.storage)?;

    let pair_id = get_next_pair_id(deps.storage)?;
    let pair_options_addr = pair_options.str_to_addr(deps.api)?;
    let pair = Pair::new(
        pair_id,
        info.sender.clone(),
        collection,
        denom,
        pair_type,
        bonding_curve,
        pair_options_addr,
    );

    // Burn the create pool fee
    let paid_amount = must_pay(&info, &sudo_params.create_pool_fee.denom)?;
    if paid_amount != sudo_params.create_pool_fee.amount {
        return Err(ContractError::InsufficientFunds {
            expected: sudo_params.create_pool_fee,
        });
    }

    let mut response = Response::new();
    response = pair.save(deps.storage, response)?;

    if sudo_params.create_pool_fee.amount > Uint128::zero() {
        response = append_fair_burn_msg(
            &sudo_params.fair_burn,
            vec![sudo_params.create_pool_fee],
            None,
            response,
        );
    }

    Ok(response)
}

/// Execute a DepositTokens message
pub fn execute_deposit_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pair_id: u64,
) -> Result<Response, ContractError> {
    let mut pair = pairs().load(deps.storage, pair_id)?;

    // Only the owner of the pair can deposit and withdraw assets
    only_pair_owner(&info, &pair)?;

    // Track the total amount of tokens that have been deposited into the pair
    let deposit_amount = must_pay(&info, &pair.denom)?;
    pair.track_token_deposit(deposit_amount)?;

    let mut response = Response::new();
    response = pair.save(deps.storage, response)?;

    // let event = pair.create_event("deposit-tokens", vec!["id", "spot_price", "total_tokens"])?;
    // response = response.add_event(event);

    Ok(response)
}

/// Execute a DepositNfts message
pub fn execute_deposit_nfts(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    pair_id: u64,
    collection: Addr,
    nft_token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pair = pairs().load(deps.storage, pair_id)?;

    // Only the owner of the pair can deposit and withdraw assets
    only_pair_owner(&info, &pair)?;

    ensure!(
        collection == pair.collection,
        ContractError::InvalidInput(format!(
            "invalid collection ({}) for pair ({})",
            collection, pair.id
        ))
    );

    // Push the NFT transfer messages
    let mut response = Response::new();
    for nft_token_id in &nft_token_ids {
        // For a pair to take an NFT deposit it must do the following:
        // 1. Append a transfer NFT message to the response
        // 2. Store the NFT token ID and pair in the NFT_DEPOSITS map
        // 3. Increment the total_nfts field on the pair

        response = response.add_submessage(transfer_nft(
            &collection,
            &nft_token_id,
            &env.contract.address,
        ));

        store_nft_deposit(deps.storage, pair.id, nft_token_id)?;
        pair.track_nft_deposit()?;
    }

    response = pair.save(deps.storage, response)?;

    // let event = Event::new("deposit-nfts").add_attributes(vec![
    //     attr("pair_id", pair.id.to_string()),
    //     attr("total_nfts", pair.total_nfts.to_string()),
    //     attr("nft_token_ids", nft_token_ids.join(",")),
    // ]);
    // response = response.add_event(event);

    Ok(response)
}

/// Execute a WithdrawTokens message
pub fn execute_withdraw_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pair_id: u64,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pair = pairs().load(deps.storage, pair_id)?;

    // Only the owner of the pair can deposit and withdraw assets
    only_pair_owner(&info, &pair)?;

    let mut response = Response::new();

    response = response.add_submessage(transfer_coin(
        coin(amount.u128(), pair.denom.clone()),
        &pair.,
    ));

    // Withdraw tokens to the asset recipient if specified, otherwise to the sender
    let recipient = asset_recipient.unwrap_or(info.sender);
    transfer_token(
        coin(amount.u128(), NATIVE_DENOM),
        recipient.as_ref(),
        &mut response,
    )?;
    // Track total amount owned by the pair
    pair.withdraw_tokens(amount)?;

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;
    response = save_pair(deps.storage, &mut pair, &marketplace_params, response)?;

    let event = pair.create_event("withdraw-tokens", vec!["id", "spot_price", "total_tokens"])?;
    response = response.add_event(event);

    Ok(response)
}

// /// Execute a WithdrawAllNfts message, a convenvience method for withdrawing all tokens
// pub fn execute_withdraw_all_tokens(
//     deps: DepsMut,
//     info: MessageInfo,
//     pair_id: u64,
//     asset_recipient: Option<Addr>,
// ) -> Result<Response, ContractError> {
//     let pair = pairs().load(deps.storage, pair_id)?;
//     execute_withdraw_tokens(deps, info, pair_id, pair.total_tokens, asset_recipient)
// }

// /// Execute a WithdrawNfts message
// pub fn execute_withdraw_nfts(
//     deps: DepsMut,
//     info: MessageInfo,
//     pair_id: u64,
//     nft_token_ids: Vec<String>,
//     asset_recipient: Option<Addr>,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let mut pair = pairs().load(deps.storage, pair_id)?;
//     // Only the owner of the pair can deposit and withdraw assets
//     only_owner(&info, &pair)?;

//     let mut response = Response::new();

//     // Track the NFTs that have been withdrawn from the pair
//     pair.withdraw_nfts(&nft_token_ids)?;

//     // Withdraw NFTs to the asset recipient if specified, otherwise to the sender
//     let recipient = asset_recipient.unwrap_or(info.sender);
//     for nft_token_id in &nft_token_ids {
//         transfer_nft(
//             nft_token_id,
//             recipient.as_ref(),
//             pair.collection.as_ref(),
//             &mut response,
//         )?;
//         remove_nft_deposit(deps.storage, pair_id, nft_token_id)?;
//     }

//     let config = CONFIG.load(deps.storage)?;
//     let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;
//     response = save_pair(deps.storage, &mut pair, &marketplace_params, response)?;

//     let event = Event::new("withdraw-nfts").add_attributes(vec![
//         attr("pair_id", pair.id.to_string()),
//         attr("total_nfts", pair.total_nfts.to_string()),
//         attr("nft_token_ids", nft_token_ids.join(",")),
//     ]);
//     response = response.add_event(event);

//     Ok(response)
// }

// /// Execute a WithdrawAllNfts message, a convenvience method for withdrawing all NFTs
// pub fn execute_withdraw_all_nfts(
//     deps: DepsMut,
//     info: MessageInfo,
//     pair_id: u64,
//     asset_recipient: Option<Addr>,
// ) -> Result<Response, ContractError> {
//     let withdrawal_batch_size: u8 = 10;
//     let token_id_response = query_pair_nft_token_ids(
//         deps.as_ref(),
//         pair_id,
//         QueryOptions {
//             descending: None,
//             start_after: None,
//             limit: Some(withdrawal_batch_size as u32),
//         },
//     )?;

//     execute_withdraw_nfts(
//         deps,
//         info,
//         pair_id,
//         token_id_response.nft_token_ids,
//         asset_recipient,
//     )
// }

// /// Execute an UpdatePairConfig message
// /// Option paramaters that are not specified will not be updated
// pub fn execute_update_pair_config(
//     deps: DepsMut,
//     info: MessageInfo,
//     pair_id: u64,
//     asset_recipient: Option<Addr>,
//     delta: Option<Uint128>,
//     spot_price: Option<Uint128>,
//     finders_fee_bps: Option<u64>,
//     swap_fee_bps: Option<u64>,
//     reinvest_tokens: Option<bool>,
//     reinvest_nfts: Option<bool>,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let mut pair = pairs().load(deps.storage, pair_id)?;
//     // Only the owner of the pair can update the pair config
//     only_owner(&info, &pair)?;

//     let mut attr_keys: Vec<&str> = vec!["id"];
//     if let Some(_asset_recipient) = asset_recipient {
//         pair.asset_recipient = Some(_asset_recipient);
//         attr_keys.push("asset_recipient");
//     }
//     if let Some(_spot_price) = spot_price {
//         pair.spot_price = _spot_price;
//         attr_keys.push("spot_price");
//     }
//     if let Some(_delta) = delta {
//         pair.delta = _delta;
//         attr_keys.push("delta");
//     }
//     if let Some(_swap_fee_bps) = swap_fee_bps {
//         pair.swap_fee_percent = Decimal::percent(_swap_fee_bps);
//         attr_keys.push("swap_fee_percent");
//     }
//     if let Some(_finders_fee_bps) = finders_fee_bps {
//         pair.finders_fee_percent = Decimal::percent(_finders_fee_bps);
//         attr_keys.push("finders_fee_percent");
//     }
//     if let Some(_reinvest_tokens) = reinvest_tokens {
//         pair.reinvest_tokens = _reinvest_tokens;
//         attr_keys.push("reinvest_tokens");
//     }
//     if let Some(_reinvest_nfts) = reinvest_nfts {
//         pair.reinvest_nfts = _reinvest_nfts;
//         attr_keys.push("reinvest_nfts");
//     }

//     let config = CONFIG.load(deps.storage)?;
//     let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

//     let mut response = Response::new();
//     response = save_pair(deps.storage, &mut pair, &marketplace_params, response)?;

//     let event = pair.create_event("update-pair-config", attr_keys)?;
//     response = response.add_event(event);

//     Ok(response)
// }

// /// Execute a SetActivePair message
// pub fn execute_set_active_pair(
//     deps: DepsMut,
//     info: MessageInfo,
//     pair_id: u64,
//     is_active: bool,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let mut pair = pairs().load(deps.storage, pair_id)?;
//     // Only the owner of the pair can update the pair config
//     only_owner(&info, &pair)?;

//     pair.set_active(is_active)?;

//     let config = CONFIG.load(deps.storage)?;
//     let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

//     let mut response = Response::new();
//     response = save_pair(deps.storage, &mut pair, &marketplace_params, response)?;

//     let event = pair.create_event("set-active-pair", vec!["id", "is_active"])?;
//     response = response.add_event(event);

//     Ok(response)
// }

// /// Execute a RemovePair message
// pub fn execute_remove_pair(
//     deps: DepsMut,
//     info: MessageInfo,
//     pair_id: u64,
//     asset_recipient: Option<Addr>,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let mut pair = pairs().load(deps.storage, pair_id)?;
//     // Only the owner of the pair can remove the pair
//     only_owner(&info, &pair)?;

//     // Pairs that hold NFTs cannot be removed
//     if pair.total_nfts > 0 {
//         return Err(ContractError::UnableToRemovePair(format!(
//             "pair {} still has NFTs",
//             pair_id
//         )));
//     }

//     let config = CONFIG.load(deps.storage)?;
//     let mut response = Response::new();

//     // If the pair has tokens, transfer them to the asset recipient
//     if pair.total_tokens > Uint128::zero() {
//         let recipient = asset_recipient.unwrap_or(info.sender);
//         transfer_token(
//             coin(pair.total_tokens.u128(), NATIVE_DENOM),
//             recipient.as_ref(),
//             &mut response,
//         )?;
//     }

//     let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;
//     response = remove_pair(deps.storage, &mut pair, &marketplace_params, response)?;

//     let event = pair.create_event("remove-pair", vec!["id"])?;
//     response = response.add_event(event);

//     Ok(response)
// }

// /// Execute a DirectSwapNftsForTokens message
// pub fn execute_direct_swap_nfts_for_tokens(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     pair_id: u64,
//     nfts_to_swap: Vec<NftSwap>,
//     swap_params: SwapParams,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let pair = pairs().load(deps.storage, pair_id)?;

//     let swap_prep_result = prep_for_swap(
//         deps.as_ref(),
//         &Some(env.block),
//         &info.sender,
//         &pair.collection,
//         &swap_params,
//     )?;

//     validate_nft_swaps_for_sell(deps.as_ref(), &info, &pair.collection, &nfts_to_swap)?;

//     let mut response = Response::new();
//     let mut pairs_to_save: Vec<Pair>;
//     let swaps: Vec<Swap>;

//     {
//         let mut processor = SwapProcessor::new(
//             TransactionType::UserSubmitsNfts,
//             env.contract.address.clone(),
//             pair.collection.clone(),
//             info.sender,
//             Uint128::zero(),
//             swap_prep_result.asset_recipient,
//             swap_prep_result
//                 .marketplace_params
//                 .params
//                 .trading_fee_percent,
//             swap_prep_result.marketplace_params.params.min_price,
//             swap_prep_result.collection_royalties,
//             swap_prep_result.finder,
//             swap_prep_result.developer,
//         );
//         processor.direct_swap_nfts_for_tokens(pair, nfts_to_swap, swap_params)?;
//         processor.finalize_transaction(&mut response)?;
//         swaps = processor.swaps;
//         pairs_to_save = processor.pairs_to_save.into_values().collect();
//     }

//     update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
//     response = save_pairs(
//         deps.storage,
//         pairs_to_save.iter_mut().collect(),
//         &swap_prep_result.marketplace_params,
//         response,
//     )?;
//     response = response.add_events(get_transaction_events(&swaps, &pairs_to_save));

//     Ok(response)
// }

// /// Execute a SwapNftsForTokens message
// pub fn execute_swap_nfts_for_tokens(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     nfts_to_swap: Vec<NftSwap>,
//     swap_params: SwapParams,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let swap_prep_result = prep_for_swap(
//         deps.as_ref(),
//         &Some(env.block),
//         &info.sender,
//         &collection,
//         &swap_params,
//     )?;

//     validate_nft_swaps_for_sell(deps.as_ref(), &info, &collection, &nfts_to_swap)?;

//     let mut response = Response::new();
//     let mut pairs_to_save: Vec<Pair>;
//     let swaps: Vec<Swap>;

//     {
//         let mut processor = SwapProcessor::new(
//             TransactionType::UserSubmitsNfts,
//             env.contract.address.clone(),
//             collection,
//             info.sender,
//             Uint128::zero(),
//             swap_prep_result.asset_recipient,
//             swap_prep_result
//                 .marketplace_params
//                 .params
//                 .trading_fee_percent,
//             swap_prep_result.marketplace_params.params.min_price,
//             swap_prep_result.collection_royalties,
//             swap_prep_result.finder,
//             swap_prep_result.developer,
//         );
//         processor.swap_nfts_for_tokens(deps.as_ref().storage, nfts_to_swap, swap_params)?;
//         processor.finalize_transaction(&mut response)?;
//         swaps = processor.swaps;
//         pairs_to_save = processor.pairs_to_save.into_values().collect();
//     }

//     update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
//     response = save_pairs(
//         deps.storage,
//         pairs_to_save.iter_mut().collect(),
//         &swap_prep_result.marketplace_params,
//         response,
//     )?;
//     response = response.add_events(get_transaction_events(&swaps, &pairs_to_save));

//     Ok(response)
// }

// /// Execute a DirectSwapTokensForSpecificNfts message
// pub fn execute_direct_swap_tokens_for_specific_nfts(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     pair_id: u64,
//     nfts_to_swap_for: Vec<NftSwap>,
//     swap_params: SwapParams,
// ) -> Result<Response, ContractError> {
//     let pair = pairs().load(deps.storage, pair_id)?;
//     execute_swap_tokens_for_specific_nfts(
//         deps,
//         env,
//         info,
//         pair.collection,
//         vec![PairNftSwap {
//             pair_id,
//             nft_swaps: nfts_to_swap_for,
//         }],
//         swap_params,
//     )
// }

// /// Execute a SwapTokensForSpecificNfts message
// pub fn execute_swap_tokens_for_specific_nfts(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     nfts_to_swap_for: Vec<PairNftSwap>,
//     swap_params: SwapParams,
// ) -> Result<Response, ContractError> {
//     let swap_prep_result = prep_for_swap(
//         deps.as_ref(),
//         &Some(env.block),
//         &info.sender,
//         &collection,
//         &swap_params,
//     )?;

//     let received_amount = validate_nft_swaps_for_buy(&info, &nfts_to_swap_for)?;

//     let mut response = Response::new();
//     let mut pairs_to_save: Vec<Pair>;
//     let swaps: Vec<Swap>;

//     {
//         let mut processor = SwapProcessor::new(
//             TransactionType::UserSubmitsTokens,
//             env.contract.address.clone(),
//             collection,
//             info.sender,
//             received_amount,
//             swap_prep_result.asset_recipient,
//             swap_prep_result
//                 .marketplace_params
//                 .params
//                 .trading_fee_percent,
//             swap_prep_result.marketplace_params.params.min_price,
//             swap_prep_result.collection_royalties,
//             swap_prep_result.finder,
//             swap_prep_result.developer,
//         );
//         processor.swap_tokens_for_specific_nfts(deps.storage, nfts_to_swap_for, swap_params)?;
//         processor.finalize_transaction(&mut response)?;
//         swaps = processor.swaps;
//         pairs_to_save = processor.pairs_to_save.into_values().collect();
//     }

//     update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
//     response = save_pairs(
//         deps.storage,
//         pairs_to_save.iter_mut().collect(),
//         &swap_prep_result.marketplace_params,
//         response,
//     )?;
//     response = response.add_events(get_transaction_events(&swaps, &pairs_to_save));

//     Ok(response)
// }

// /// Execute a SwapTokensForAnyNfts message
// pub fn execute_swap_tokens_for_any_nfts(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     max_expected_token_input: Vec<Uint128>,
//     swap_params: SwapParams,
// ) -> Result<Response, ContractError> {
//     let swap_prep_result = prep_for_swap(
//         deps.as_ref(),
//         &Some(env.block),
//         &info.sender,
//         &collection,
//         &swap_params,
//     )?;

//     if max_expected_token_input.is_empty() {
//         return Err(ContractError::InvalidInput(
//             "max expected token input must not be empty".to_string(),
//         ));
//     }

//     // User must send enough tokens to cover the swap
//     // Should be the sum of all the token amounts in max_expected_token_input
//     let received_amount = must_pay(&info, NATIVE_DENOM)?;
//     let expected_amount = max_expected_token_input
//         .iter()
//         .fold(Uint128::zero(), |acc, amount| acc + amount);
//     if received_amount < expected_amount {
//         return Err(ContractError::InsufficientFunds(format!(
//             "expected {} but received {}",
//             expected_amount, received_amount
//         )));
//     }

//     let mut response = Response::new();
//     let mut pairs_to_save: Vec<Pair>;
//     let swaps: Vec<Swap>;

//     {
//         let mut processor = SwapProcessor::new(
//             TransactionType::UserSubmitsTokens,
//             env.contract.address.clone(),
//             collection,
//             info.sender,
//             received_amount,
//             swap_prep_result.asset_recipient,
//             swap_prep_result
//                 .marketplace_params
//                 .params
//                 .trading_fee_percent,
//             swap_prep_result.marketplace_params.params.min_price,
//             swap_prep_result.collection_royalties,
//             swap_prep_result.finder,
//             swap_prep_result.developer,
//         );
//         processor.swap_tokens_for_any_nfts(deps.storage, max_expected_token_input, swap_params)?;
//         processor.finalize_transaction(&mut response)?;
//         swaps = processor.swaps;
//         pairs_to_save = processor.pairs_to_save.into_values().collect();
//     }

//     update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
//     response = save_pairs(
//         deps.storage,
//         pairs_to_save.iter_mut().collect(),
//         &swap_prep_result.marketplace_params,
//         response,
//     )?;
//     response = response.add_events(get_transaction_events(&swaps, &pairs_to_save));

//     Ok(response)
// }
