use crate::state::{buy_pool_quotes, pools, sell_pool_quotes, Pool, PoolQuote, POOL_COUNTER};
use crate::ContractError;
use cosmwasm_std::{
    to_binary, Addr, Attribute, BankMsg, BlockInfo, Coin, Deps, MessageInfo, Order, StdResult,
    Storage, SubMsg, Timestamp, WasmMsg,
};
use sg721::RoyaltyInfoResponse;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg721_base::ExecuteMsg as Sg721ExecuteMsg;
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::Response;

pub fn load_marketplace_params(
    deps: Deps,
    marketplace_addr: &Addr,
) -> Result<ParamsResponse, ContractError> {
    let marketplace_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(marketplace_addr, &MarketplaceQueryMsg::Params {})?;
    Ok(marketplace_params)
}

pub fn load_collection_royalties(
    deps: Deps,
    collection_addr: &Addr,
) -> Result<Option<RoyaltyInfoResponse>, ContractError> {
    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(collection_addr, &Sg721QueryMsg::CollectionInfo {})?;
    Ok(collection_info.royalty_info)
}

pub fn get_next_pool_counter(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let pool_counter = POOL_COUNTER.load(store)?;
    POOL_COUNTER.save(store, &(pool_counter + 1))?;
    Ok(pool_counter)
}

pub fn update_pool_quotes(store: &mut dyn Storage, pool: &Pool) -> Result<(), ContractError> {
    if pool.can_buy_nfts() {
        if !pool.is_active {
            buy_pool_quotes().remove(store, pool.id)?;
        } else {
            if let Some(_buy_price_quote) = pool.get_buy_quote()? {
                buy_pool_quotes().save(
                    store,
                    pool.id,
                    &PoolQuote {
                        id: pool.id,
                        collection: pool.collection.clone(),
                        quote_price: _buy_price_quote,
                    },
                )?;
            } else {
                buy_pool_quotes().remove(store, pool.id)?;
            }
        }
    }
    if pool.can_sell_nfts() {
        if !pool.is_active {
            sell_pool_quotes().remove(store, pool.id)?;
        } else {
            if let Some(_sell_price_quote) = pool.get_sell_quote()? {
                sell_pool_quotes().save(
                    store,
                    pool.id,
                    &PoolQuote {
                        id: pool.id,
                        collection: pool.collection.clone(),
                        quote_price: _sell_price_quote,
                    },
                )?;
            } else {
                sell_pool_quotes().remove(store, pool.id)?;
            }
        }
    }

    Ok(())
}

pub fn save_pool(store: &mut dyn Storage, pool: &Pool) -> Result<(), ContractError> {
    pool.validate()?;
    update_pool_quotes(store, pool)?;
    pools().save(store, pool.id, pool)?;

    Ok(())
}

pub fn save_pools(store: &mut dyn Storage, pools: Vec<Pool>) -> Result<(), ContractError> {
    for pool in pools {
        save_pool(store, &pool)?;
    }
    Ok(())
}

pub fn remove_pool(store: &mut dyn Storage, pool: &mut Pool) -> Result<(), ContractError> {
    pool.set_active(false)?;
    update_pool_quotes(store, pool)?;
    pools().remove(store, pool.id)?;

    Ok(())
}

pub fn get_pool_attributes(pool: &Pool) -> Vec<Attribute> {
    vec![
        Attribute {
            key: "id".to_string(),
            value: pool.id.to_string(),
        },
        Attribute {
            key: "collection".to_string(),
            value: pool.collection.to_string(),
        },
        Attribute {
            key: "owner".to_string(),
            value: pool.owner.to_string(),
        },
        Attribute {
            key: "asset_recipient".to_string(),
            value: pool
                .asset_recipient
                .clone()
                .map_or("None".to_string(), |addr| addr.to_string()),
        },
        Attribute {
            key: "pool_type".to_string(),
            value: pool.pool_type.to_string(),
        },
        Attribute {
            key: "bonding_curve".to_string(),
            value: pool.bonding_curve.to_string(),
        },
        Attribute {
            key: "spot_price".to_string(),
            value: pool.spot_price.to_string(),
        },
        Attribute {
            key: "delta".to_string(),
            value: pool.delta.to_string(),
        },
        Attribute {
            key: "total_tokens".to_string(),
            value: pool.total_tokens.to_string(),
        },
        Attribute {
            key: "nft_token_ids".to_string(),
            value: [
                "[".to_string(),
                pool.nft_token_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
                "]".to_string(),
            ]
            .join(""),
        },
        Attribute {
            key: "fee_bps".to_string(),
            value: pool
                .fee_bps
                .clone()
                .map_or("None".to_string(), |f| f.to_string()),
        },
    ]
}

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

pub fn transfer_token(coin_send: Coin, recipient: &str, response: &mut Response) -> StdResult<()> {
    let token_transfer_msg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![coin_send],
    };
    response.messages.push(SubMsg::new(token_transfer_msg));

    Ok(())
}

pub fn only_owner(info: &MessageInfo, pool: &Pool) -> Result<(), ContractError> {
    if pool.owner != info.sender {
        return Err(ContractError::Unauthorized(String::from(
            "sender is not the owner of the pool",
        )));
    }
    Ok(())
}

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

pub fn check_deadline(block: &BlockInfo, deadline: Timestamp) -> Result<(), ContractError> {
    if deadline <= block.time {
        return Err(ContractError::DeadlinePassed);
    }
    Ok(())
}
