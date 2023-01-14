use crate::state::{Pool};
use crate::ContractError;
use crate::state::{POOL_COUNTER, pools, PoolType, PoolQuote, buy_pool_quotes, sell_pool_quotes};
use sg_std::Response;
use cosmwasm_std::{
    Storage, Attribute, Addr, StdResult, Event, WasmMsg, SubMsg, MessageInfo, Coin, BankMsg,
    Order, to_binary,
};
use sg721_base::ExecuteMsg as Sg721ExecuteMsg;

pub fn get_next_pool_counter(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let pool_counter = POOL_COUNTER.load(store)?;
    POOL_COUNTER.save(store, &(pool_counter + 1))?;
    Ok(pool_counter)
}

pub fn update_pool_quotes(store: &mut dyn Storage, pool: &Pool) -> Result<(), ContractError> {
    if !pool.is_active {
        if pool.can_buy_nfts() {
            buy_pool_quotes().remove(store, pool.id)?;
        }
        if pool.can_sell_nfts() {
            sell_pool_quotes().remove(store, pool.id)?;
        }
        return Ok(())
    }

    match pool.pool_type {
        PoolType::Token => {
            if pool.total_tokens < pool.spot_price {
                buy_pool_quotes().remove(store, pool.id)?;
            } else {
                buy_pool_quotes().save(store, pool.id, &PoolQuote {
                    id: pool.id,
                    collection: pool.collection.clone(),
                    quote_price: pool.spot_price,
                })?;
            }
        }
        PoolType::Nft => {
            if pool.nft_token_ids.is_empty() {
                sell_pool_quotes().remove(store, pool.id)?;
            } else {
                sell_pool_quotes().save(store, pool.id, &PoolQuote {
                    id: pool.id,
                    collection: pool.collection.clone(),
                    quote_price: pool.spot_price,
                })?;
            }
        }
        PoolType::Trade => {
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

pub fn remove_pool(store: &mut dyn Storage, pool: &mut Pool) -> Result<(), ContractError> {
    pool.set_active(false)?;
    update_pool_quotes(store, pool)?;
    pools().remove(store, pool.id);

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
            value: pool.asset_recipient.clone().map_or("None".to_string(), |addr| addr.to_string()),
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
            value: pool.nft_token_ids.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(","),
        },
        Attribute {
            key: "fee_bps".to_string(),
            value: pool.fee_bps.to_string(),
        },
    ]
}

pub fn transfer_nft(token_id: &String, recipient: &Addr, collection: &Addr, response: &mut Response,) -> StdResult<()> {
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

    let event = Event::new("transfer-nft")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("recipient", recipient.to_string());
    response.events.push(event);
    
    Ok(())
}

pub fn transfer_token(coin_send: Coin, recipient: String, event_label: &str, response: &mut Response) -> StdResult<()> {
    let token_transfer_msg = BankMsg::Send {
        to_address: recipient.clone(),
        amount: vec![coin_send.clone()]
    };
    response.messages.push(SubMsg::new(token_transfer_msg));

    let event = Event::new(event_label)
        .add_attribute("coin", coin_send.to_string())
        .add_attribute("recipient", recipient.to_string());
    response.events.push(event);

    Ok(())
}

pub fn only_owner(
    info: &MessageInfo,
    pool: &Pool
) -> Result<(), ContractError> {
    if pool.owner != info.sender {
        return Err(ContractError::Unauthorized(String::from("sender is not the owner of the pool")));
    }
    Ok(())
}

pub fn option_bool_to_order(descending: Option<bool>) -> Order {
    match descending {
       Some(_descending) => if _descending { Order::Descending } else { Order::Ascending },
       _ => Order::Ascending
   }
}