use crate::msg::{NftSwap, PoolNftSwap, SwapParams, TransactionType};
use crate::state::{
    buy_pool_quotes, pools, sell_pool_quotes, BondingCurve, Pool, PoolQuote, CONFIG, NFT_DEPOSITS,
    POOL_COUNTER,
};
use crate::swap_processor::Swap;
use crate::ContractError;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, BlockInfo, Coin, Deps, Empty, Event, MessageInfo, Order, StdResult,
    Storage, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw721::OwnerOfResponse;
use cw721_base::helpers::Cw721Contract;
use cw_utils::{maybe_addr, must_pay};
use sg721::RoyaltyInfoResponse;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg721_base::ExecuteMsg as Sg721ExecuteMsg;
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::Response;
use std::collections::BTreeSet;
use std::marker::PhantomData;

/// Load the marketplace params for use within the contract
pub fn load_marketplace_params(
    deps: Deps,
    marketplace_addr: &Addr,
) -> Result<ParamsResponse, ContractError> {
    let marketplace_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(marketplace_addr, &MarketplaceQueryMsg::Params {})?;
    Ok(marketplace_params)
}

/// Load the collection royalties as defined on the NFT collection contract
pub fn load_collection_royalties(
    deps: Deps,
    collection_addr: &Addr,
) -> Result<Option<RoyaltyInfoResponse>, ContractError> {
    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(collection_addr, &Sg721QueryMsg::CollectionInfo {})?;
    Ok(collection_info.royalty_info)
}

/// Retrieve the next pool counter from storage and increment it
pub fn get_next_pool_counter(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let pool_counter = POOL_COUNTER.load(store)?;
    POOL_COUNTER.save(store, &(pool_counter + 1))?;
    Ok(pool_counter)
}

pub fn remove_buy_pool_quote(
    store: &mut dyn Storage,
    pool_id: u64,
    response: Response,
) -> Result<Response, ContractError> {
    let old_data = buy_pool_quotes().may_load(store, pool_id)?;
    if old_data.is_none() {
        return Ok(response);
    }
    buy_pool_quotes().replace(store, pool_id, None, old_data.as_ref())?;
    let response = response
        .add_event(Event::new("remove-buy-pool-quote").add_attribute("id", pool_id.to_string()));
    Ok(response)
}

pub fn remove_sell_pool_quote(
    store: &mut dyn Storage,
    pool_id: u64,
    response: Response,
) -> Result<Response, ContractError> {
    let old_data = sell_pool_quotes().may_load(store, pool_id)?;
    if old_data.is_none() {
        return Ok(response);
    }
    sell_pool_quotes().replace(store, pool_id, None, old_data.as_ref())?;
    let response = response
        .add_event(Event::new("remove-sell-pool-quote").add_attribute("id", pool_id.to_string()));
    Ok(response)
}

/// Update the indexed buy pool quotes for a specific pool
pub fn update_buy_pool_quotes(
    store: &mut dyn Storage,
    pool: &Pool,
    min_price: Uint128,
    response: Response,
) -> Result<Response, ContractError> {
    if !pool.can_buy_nfts() {
        return Ok(response);
    }
    let mut response = response;
    if !pool.is_active {
        response = remove_buy_pool_quote(store, pool.id, response)?;
        return Ok(response);
    }
    let buy_pool_quote = pool.get_buy_quote(min_price)?;

    // If the pool quote is less than the minimum price, remove it from the index
    if buy_pool_quote.is_none() {
        response = remove_buy_pool_quote(store, pool.id, response)?;
        return Ok(response);
    }
    let pool_quote = PoolQuote {
        id: pool.id,
        collection: pool.collection.clone(),
        quote_price: buy_pool_quote.unwrap(),
    };
    buy_pool_quotes().save(store, pool.id, &pool_quote)?;
    response = response.add_event(
        Event::new("add-buy-pool-quote")
            .add_attribute("id", pool_quote.id.to_string())
            .add_attribute("collection", pool_quote.collection.to_string())
            .add_attribute("quote_price", pool_quote.quote_price.to_string()),
    );
    Ok(response)
}

/// Update the indexed sell pool quotes for a specific pool
pub fn update_sell_pool_quotes(
    store: &mut dyn Storage,
    pool: &Pool,
    min_price: Uint128,
    response: Response,
) -> Result<Response, ContractError> {
    if !pool.can_sell_nfts() {
        return Ok(response);
    }
    let mut response = response;
    if !pool.is_active {
        response = remove_sell_pool_quote(store, pool.id, response)?;
        return Ok(response);
    }
    let sell_pool_quote = pool.get_sell_quote(min_price)?;
    // If the pool quote is less than the minimum price, remove it from the index
    if sell_pool_quote.is_none() {
        response = remove_sell_pool_quote(store, pool.id, response)?;
        return Ok(response);
    }
    let pool_quote = PoolQuote {
        id: pool.id,
        collection: pool.collection.clone(),
        quote_price: sell_pool_quote.unwrap(),
    };
    sell_pool_quotes().save(store, pool.id, &pool_quote)?;
    response = response.add_event(
        Event::new("add-sell-pool-quote")
            .add_attribute("id", pool_quote.id.to_string())
            .add_attribute("collection", pool_quote.collection.to_string())
            .add_attribute("quote_price", pool_quote.quote_price.to_string()),
    );
    Ok(response)
}

/// Force pool property values for certain pools
pub fn force_property_values(pool: &mut Pool) -> Result<(), ContractError> {
    if pool.bonding_curve == BondingCurve::ConstantProduct {
        pool.delta = Uint128::zero();
        if pool.total_nfts == 0u64 {
            pool.spot_price = Uint128::zero();
        } else {
            pool.spot_price = pool
                .total_tokens
                .checked_div(Uint128::from(pool.total_nfts))
                .unwrap();
        }
    };
    Ok(())
}

/// Save a pool, check invariants, update pool quotes
/// IMPORTANT: this function must always be called when saving a pool!
pub fn save_pool(
    store: &mut dyn Storage,
    pool: &mut Pool,
    marketplace_params: &ParamsResponse,
    response: Response,
) -> Result<Response, ContractError> {
    let mut response = response;
    pool.validate(marketplace_params)?;
    force_property_values(pool)?;
    response = update_buy_pool_quotes(store, pool, marketplace_params.params.min_price, response)?;
    response = update_sell_pool_quotes(store, pool, marketplace_params.params.min_price, response)?;
    pools().save(store, pool.id, pool)?;

    Ok(response)
}

/// Save pools batch convenience function
pub fn save_pools(
    store: &mut dyn Storage,
    pools: Vec<&mut Pool>,
    marketplace_params: &ParamsResponse,
    response: Response,
) -> Result<Response, ContractError> {
    let mut response = response;
    for pool in pools {
        response = save_pool(store, pool, marketplace_params, response)?;
    }
    Ok(response)
}

/// Remove a pool, and remove pool quotes
/// IMPORTANT: this function must always be called when removing a pool!
pub fn remove_pool(
    store: &mut dyn Storage,
    pool: &mut Pool,
    marketplace_params: &ParamsResponse,
    response: Response,
) -> Result<Response, ContractError> {
    let mut response = response;
    pool.set_active(false)?;
    response = update_buy_pool_quotes(store, pool, marketplace_params.params.min_price, response)?;
    response = update_sell_pool_quotes(store, pool, marketplace_params.params.min_price, response)?;
    pools().remove(store, pool.id)?;

    Ok(response)
}

/// Store NFT deposit
pub fn store_nft_deposit(
    storage: &mut dyn Storage,
    pool_id: u64,
    nft_token_id: &str,
) -> StdResult<()> {
    NFT_DEPOSITS.save(storage, (pool_id, nft_token_id.to_string()), &true)
}

/// Remove NFT deposit
pub fn remove_nft_deposit(storage: &mut dyn Storage, pool_id: u64, nft_token_id: &str) {
    NFT_DEPOSITS.remove(storage, (pool_id, nft_token_id.to_string()))
}

/// Verify NFT is deposited into pool
pub fn verify_nft_deposit(
    storage: &dyn Storage,
    pool_id: u64,
    nft_token_id: &str,
) -> Result<bool, ContractError> {
    let result = NFT_DEPOSITS.may_load(storage, (pool_id, nft_token_id.to_string()))?;
    Ok(result.is_some())
}

/// Grab the first NFT in a pool
pub fn get_nft_deposit(
    storage: &dyn Storage,
    pool_id: u64,
    offset: u32,
) -> Result<Option<String>, ContractError> {
    let mut nft_token_id: Vec<String> = NFT_DEPOSITS
        .prefix(pool_id)
        .range(storage, None, None, Order::Ascending)
        .skip(offset as usize)
        .take(1)
        .map(|item| item.map(|(nft_token_id, _)| nft_token_id))
        .collect::<StdResult<_>>()?;
    Ok(nft_token_id.pop())
}

/// Process swaps for NFT deposit changes
pub fn update_nft_deposits(
    storage: &mut dyn Storage,
    contract: &Addr,
    swaps: &Vec<Swap>,
) -> Result<(), ContractError> {
    for swap in swaps.iter() {
        match swap.transaction_type {
            TransactionType::NftsForTokens => {
                if &swap.nft_payment.address == contract {
                    store_nft_deposit(storage, swap.pool_id, &swap.nft_payment.nft_token_id)?;
                }
            }
            TransactionType::TokensForNfts => {
                remove_nft_deposit(storage, swap.pool_id, &swap.nft_payment.nft_token_id)
            }
        }
    }
    Ok(())
}

/// Push the transfer NFT message on the NFT collection contract
pub fn transfer_nft(
    token_id: &str,
    recipient: &str,
    collection: &str,
    response: &mut Response,
) -> StdResult<()> {
    let sg721_transfer_msg = Sg721ExecuteMsg::TransferNft {
        token_id: token_id.to_string(),
        recipient: recipient.to_string(),
    };

    let exec_sg721_transfer = SubMsg::new(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&sg721_transfer_msg)?,
        funds: vec![],
    });
    response.messages.push(exec_sg721_transfer);
    Ok(())
}

/// Push the BankeMsg send message
pub fn transfer_token(coin_send: Coin, recipient: &str, response: &mut Response) -> StdResult<()> {
    let token_transfer_msg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![coin_send],
    };
    response.messages.push(SubMsg::new(token_transfer_msg));

    Ok(())
}

/// Verify that a message is indeed invoked by the owner
pub fn only_owner(info: &MessageInfo, pool: &Pool) -> Result<(), ContractError> {
    if pool.owner != info.sender {
        return Err(ContractError::Unauthorized(String::from(
            "sender is not the owner of the pool",
        )));
    }
    Ok(())
}

pub fn only_nft_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<OwnerOfResponse, ContractError> {
    let res = Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::Unauthorized(String::from(
            "only the owner can call this function",
        )));
    }

    Ok(res)
}

/// Convert an option bool to an Order
pub fn option_bool_to_order(descending: Option<bool>, default: Order) -> Order {
    match descending {
        Some(_descending) => {
            if _descending {
                Order::Descending
            } else {
                Order::Ascending
            }
        }
        _ => default,
    }
}

/// Verify that a message has been processed before the specified deadline
pub fn check_deadline(block: &BlockInfo, deadline: Timestamp) -> Result<(), ContractError> {
    if deadline <= block.time {
        return Err(ContractError::DeadlinePassed);
    }
    Ok(())
}

/// Verify that the finder address is neither the sender nor the asset recipient
pub fn validate_finder(
    finder: &Option<Addr>,
    sender: &Addr,
    asset_recipient: &Option<Addr>,
) -> Result<(), ContractError> {
    if finder.is_none() {
        return Ok(());
    }
    if finder.as_ref().unwrap() == sender || finder == asset_recipient {
        return Err(ContractError::InvalidInput("finder is invalid".to_string()));
    }
    Ok(())
}

pub struct SwapPrepResult {
    pub denom: String,
    pub marketplace_params: ParamsResponse,
    pub collection_royalties: Option<RoyaltyInfoResponse>,
    pub asset_recipient: Addr,
    pub finder: Option<Addr>,
    pub developer: Option<Addr>,
}

/// Prepare the contract for a swap transaction
pub fn prep_for_swap(
    deps: Deps,
    block_info: &Option<BlockInfo>,
    sender: &Addr,
    collection: &Addr,
    swap_params: &SwapParams,
) -> Result<SwapPrepResult, ContractError> {
    if let Some(_block_info) = block_info {
        check_deadline(_block_info, swap_params.deadline)?;
    }

    let finder = maybe_addr(deps.api, swap_params.finder.clone())?;
    let asset_recipient = maybe_addr(deps.api, swap_params.asset_recipient.clone())?;

    validate_finder(&finder, sender, &asset_recipient)?;

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps, &config.marketplace_addr)?;

    let collection_royalties = load_collection_royalties(deps, collection)?;

    let seller_recipient = asset_recipient.unwrap_or_else(|| sender.clone());

    Ok(SwapPrepResult {
        denom: config.denom.clone(),
        marketplace_params,
        collection_royalties,
        asset_recipient: seller_recipient,
        finder,
        developer: config.developer,
    })
}

/// Validate NftSwap vector token amounts, and NFT ownership
pub fn validate_nft_swaps_for_sell(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    nft_swaps: &[NftSwap],
) -> Result<(), ContractError> {
    if nft_swaps.is_empty() {
        return Err(ContractError::InvalidInput(
            "nft swaps must not be empty".to_string(),
        ));
    }
    let mut uniq_nft_token_ids: BTreeSet<String> = BTreeSet::new();
    for (idx, nft_swap) in nft_swaps.iter().enumerate() {
        only_nft_owner(deps, info, collection, &nft_swap.nft_token_id)?;
        if uniq_nft_token_ids.contains(&nft_swap.nft_token_id) {
            return Err(ContractError::InvalidInput(
                "found duplicate nft token id".to_string(),
            ));
        }
        uniq_nft_token_ids.insert(nft_swap.nft_token_id.clone());

        if idx == 0 {
            continue;
        }
        if nft_swaps[idx - 1].token_amount < nft_swap.token_amount {
            return Err(ContractError::InvalidInput(
                "nft swap token amounts must decrease monotonically".to_string(),
            ));
        }
    }
    Ok(())
}

/// Validate NftSwap vector token amounts, and that user has provided enough tokens
pub fn validate_nft_swaps_for_buy(
    info: &MessageInfo,
    denom: &str,
    pool_nft_swaps: &Vec<PoolNftSwap>,
) -> Result<Uint128, ContractError> {
    if pool_nft_swaps.is_empty() {
        return Err(ContractError::InvalidInput(
            "pool nft swaps must not be empty".to_string(),
        ));
    }
    let mut expected_amount = Uint128::zero();
    let mut uniq_nft_token_ids: BTreeSet<String> = BTreeSet::new();

    for pool_nft_swap in pool_nft_swaps {
        for (idx, nft_swap) in pool_nft_swap.nft_swaps.iter().enumerate() {
            if uniq_nft_token_ids.contains(&nft_swap.nft_token_id) {
                return Err(ContractError::InvalidInput(
                    "found duplicate nft token id".to_string(),
                ));
            }
            uniq_nft_token_ids.insert(nft_swap.nft_token_id.clone());

            expected_amount += nft_swap.token_amount;
            if idx == 0 {
                continue;
            }
            if pool_nft_swap.nft_swaps[idx - 1].token_amount > nft_swap.token_amount {
                return Err(ContractError::InvalidInput(
                    "nft swap token amounts must increase monotonically".to_string(),
                ));
            }
        }
    }

    let received_amount = must_pay(info, denom)?;
    if received_amount != expected_amount {
        return Err(ContractError::InsufficientFunds(format!(
            "expected {} but received {}",
            expected_amount, received_amount
        )));
    }
    Ok(received_amount)
}
