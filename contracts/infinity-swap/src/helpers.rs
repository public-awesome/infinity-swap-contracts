use crate::msg::{NftSwap, PairNftSwap, SwapParams};
use crate::state::{pairs, BondingCurve, Pair, NFT_DEPOSITS, PAIR_COUNTER};
use crate::ContractError;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, BlockInfo, Coin, Deps, Empty, Event, MessageInfo, StdResult, Storage,
    SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw721::OwnerOfResponse;
use cw721_base::helpers::Cw721Contract;
use cw_utils::{maybe_addr, must_pay};
use sg721::RoyaltyInfoResponse;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg721_base::ExecuteMsg as Sg721ExecuteMsg;
use sg_std::{Response, NATIVE_DENOM};
use std::collections::BTreeSet;
use std::marker::PhantomData;

/// Retrieve the next pair counter from storage and increment it
pub fn get_next_pair_id(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let pair_counter = PAIR_COUNTER.load(store)? + 1;
    PAIR_COUNTER.save(store, &pair_counter)?;
    Ok(pair_counter)
}

/// Verify that a message is indeed invoked by the owner
pub fn only_pair_owner(info: &MessageInfo, pair: &Pair) -> Result<(), ContractError> {
    if pair.owner != info.sender {
        return Err(ContractError::Unauthorized(
            "sender is not the owner of the pair".to_string(),
        ));
    }
    Ok(())
}

/// Store NFT deposit
pub fn store_nft_deposit(
    storage: &mut dyn Storage,
    pair_id: u64,
    nft_token_id: &str,
) -> StdResult<()> {
    NFT_DEPOSITS.save(storage, (pair_id, nft_token_id.to_string()), &true)
}

// pub fn remove_buy_from_pair_quote(
//     store: &mut dyn Storage,
//     pair_id: u64,
//     response: Response,
// ) -> Result<Response, ContractError> {
//     let old_data = buy_from_pair_quotes().may_load(store, pair_id)?;
//     if old_data.is_none() {
//         return Ok(response);
//     }
//     buy_from_pair_quotes().replace(store, pair_id, None, old_data.as_ref())?;
//     let response = response
//         .add_event(Event::new("remove-buy-pair-quote").add_attribute("id", pair_id.to_string()));
//     Ok(response)
// }

// pub fn remove_sell_to_pair_quote(
//     store: &mut dyn Storage,
//     pair_id: u64,
//     response: Response,
// ) -> Result<Response, ContractError> {
//     let old_data = sell_to_pair_quotes().may_load(store, pair_id)?;
//     if old_data.is_none() {
//         return Ok(response);
//     }
//     sell_to_pair_quotes().replace(store, pair_id, None, old_data.as_ref())?;
//     let response = response
//         .add_event(Event::new("remove-sell-pair-quote").add_attribute("id", pair_id.to_string()));
//     Ok(response)
// }

// /// Update the indexed buy pair quotes for a specific pair
// pub fn update_buy_from_pair_quotes(
//     store: &mut dyn Storage,
//     pair: &Pair,
//     min_price: Uint128,
//     response: Response,
// ) -> Result<Response, ContractError> {
//     if !pair.can_sell_nfts() {
//         return Ok(response);
//     }
//     let mut response = response;
//     if !pair.is_active {
//         response = remove_buy_from_pair_quote(store, pair.id, response)?;
//         return Ok(response);
//     }
//     let buy_pair_quote = pair.get_buy_from_pair_quote(min_price)?;

//     // If the pair quote is less than the minimum price, remove it from the index
//     if buy_pair_quote.is_none() {
//         response = remove_buy_from_pair_quote(store, pair.id, response)?;
//         return Ok(response);
//     }
//     let pair_quote = PairQuote {
//         id: pair.id,
//         collection: pair.collection.clone(),
//         quote_price: buy_pair_quote.unwrap(),
//     };
//     buy_from_pair_quotes().save(store, pair.id, &pair_quote)?;
//     response = response.add_event(
//         Event::new("add-buy-pair-quote")
//             .add_attribute("id", pair_quote.id.to_string())
//             .add_attribute("collection", pair_quote.collection.to_string())
//             .add_attribute("quote_price", pair_quote.quote_price.to_string()),
//     );
//     Ok(response)
// }

// /// Update the indexed sell pair quotes for a specific pair
// pub fn update_sell_to_pair_quotes(
//     store: &mut dyn Storage,
//     pair: &Pair,
//     min_price: Uint128,
//     response: Response,
// ) -> Result<Response, ContractError> {
//     if !pair.can_buy_nfts() {
//         return Ok(response);
//     }
//     let mut response = response;
//     if !pair.is_active {
//         response = remove_sell_to_pair_quote(store, pair.id, response)?;
//         return Ok(response);
//     }
//     let sell_pair_quote = pair.get_sell_to_pair_quote(min_price)?;
//     // If the pair quote is less than the minimum price, remove it from the index
//     if sell_pair_quote.is_none() {
//         response = remove_sell_to_pair_quote(store, pair.id, response)?;
//         return Ok(response);
//     }
//     let pair_quote = PairQuote {
//         id: pair.id,
//         collection: pair.collection.clone(),
//         quote_price: sell_pair_quote.unwrap(),
//     };
//     sell_to_pair_quotes().save(store, pair.id, &pair_quote)?;
//     response = response.add_event(
//         Event::new("add-sell-pair-quote")
//             .add_attribute("id", pair_quote.id.to_string())
//             .add_attribute("collection", pair_quote.collection.to_string())
//             .add_attribute("quote_price", pair_quote.quote_price.to_string()),
//     );
//     Ok(response)
// }

// /// Save pairs batch convenience function
// pub fn save_pairs(
//     store: &mut dyn Storage,
//     pairs: Vec<&mut Pair>,
//     marketplace_params: &ParamsResponse,
//     response: Response,
// ) -> Result<Response, ContractError> {
//     let mut response = response;
//     for pair in pairs {
//         response = save_pair(store, pair, marketplace_params, response)?;
//     }
//     Ok(response)
// }

// /// Remove a pair, and remove pair quotes
// /// IMPORTANT: this function must always be called when removing a pair!
// pub fn remove_pair(
//     store: &mut dyn Storage,
//     pair: &mut Pair,
//     marketplace_params: &ParamsResponse,
//     response: Response,
// ) -> Result<Response, ContractError> {
//     let mut response = response;
//     pair.set_active(false)?;
//     response =
//         update_buy_from_pair_quotes(store, pair, marketplace_params.params.min_price, response)?;
//     response =
//         update_sell_to_pair_quotes(store, pair, marketplace_params.params.min_price, response)?;
//     pairs().remove(store, pair.id)?;

//     Ok(response)
// }

// /// Remove NFT deposit
// pub fn remove_nft_deposit(
//     storage: &mut dyn Storage,
//     pair_id: u64,
//     nft_token_id: &str,
// ) -> Result<(), ContractError> {
//     let nft_found = verify_nft_deposit(storage, pair_id, nft_token_id);
//     if !nft_found {
//         return Err(ContractError::NftNotFound(nft_token_id.to_string()));
//     }
//     NFT_DEPOSITS.remove(storage, (pair_id, nft_token_id.to_string()));
//     Ok(())
// }

// /// Verify NFT is deposited into pair
// pub fn verify_nft_deposit(storage: &dyn Storage, pair_id: u64, nft_token_id: &str) -> bool {
//     NFT_DEPOSITS.has(storage, (pair_id, nft_token_id.to_string()))
// }

// /// Grab the first NFT in a pair
// pub fn get_nft_deposit(
//     storage: &dyn Storage,
//     pair_id: u64,
//     offset: u32,
// ) -> Result<Option<String>, ContractError> {
//     let mut nft_token_id: Vec<String> = NFT_DEPOSITS
//         .prefix(pair_id)
//         .range(storage, None, None, Pair::Ascending)
//         .skip(offset as usize)
//         .take(1)
//         .map(|item| item.map(|(nft_token_id, _)| nft_token_id))
//         .collect::<StdResult<_>>()?;
//     Ok(nft_token_id.pop())
// }

// /// Process swaps for NFT deposit changes
// pub fn update_nft_deposits(
//     storage: &mut dyn Storage,
//     contract: &Addr,
//     swaps: &[Swap],
// ) -> Result<(), ContractError> {
//     for swap in swaps.iter() {
//         match swap.transaction_type {
//             TransactionType::UserSubmitsNfts => {
//                 if &swap.nft_payment.address == contract {
//                     store_nft_deposit(storage, swap.pair_id, &swap.nft_payment.nft_token_id)?;
//                 }
//             }
//             TransactionType::UserSubmitsTokens => {
//                 remove_nft_deposit(storage, swap.pair_id, &swap.nft_payment.nft_token_id)?;
//             }
//         }
//     }
//     Ok(())
// }

// /// Process swaps for NFT deposit changes
// pub fn get_transaction_events(swaps: &[Swap], pairs_to_save: &[Pair]) -> Vec<Event> {
//     let mut events: Vec<Event> = vec![];
//     for swap in swaps.iter() {
//         events.push(swap.into());
//     }
//     for pair in pairs_to_save.iter() {
//         events.push(
//             pair.create_event(
//                 "pair-swap-update",
//                 vec![
//                     "id",
//                     "spot_price",
//                     "total_nfts",
//                     "total_tokens",
//                     "is_active",
//                 ],
//             )
//             .unwrap(),
//         );
//     }
//     events
// }

// /// Push the transfer NFT message on the NFT collection contract
// pub fn transfer_nft(
//     token_id: &str,
//     recipient: &str,
//     collection: &str,
//     response: &mut Response,
// ) -> StdResult<()> {
//     let sg721_transfer_msg = Sg721ExecuteMsg::TransferNft {
//         token_id: token_id.to_string(),
//         recipient: recipient.to_string(),
//     };

//     let exec_sg721_transfer = SubMsg::new(WasmMsg::Execute {
//         contract_addr: collection.to_string(),
//         msg: to_binary(&sg721_transfer_msg)?,
//         funds: vec![],
//     });
//     response.messages.push(exec_sg721_transfer);
//     Ok(())
// }

// /// Push the BankeMsg send message
// pub fn transfer_token(coin_send: Coin, recipient: &str, response: &mut Response) -> StdResult<()> {
//     let token_transfer_msg = BankMsg::Send {
//         to_address: recipient.to_string(),
//         amount: vec![coin_send],
//     };
//     response.messages.push(SubMsg::new(token_transfer_msg));

//     Ok(())
// }

// pub fn only_nft_owner(
//     deps: Deps,
//     info: &MessageInfo,
//     collection: &Addr,
//     token_id: &str,
// ) -> Result<OwnerOfResponse, ContractError> {
//     let res = Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
//         .owner_of(&deps.querier, token_id, false)?;
//     if res.owner != info.sender {
//         return Err(ContractError::Unauthorized(String::from(
//             "only the owner can call this function",
//         )));
//     }

//     Ok(res)
// }

// /// Convert an option bool to an Pair
// pub fn option_bool_to_pair(descending: Option<bool>, default: Pair) -> Pair {
//     match descending {
//         Some(_descending) => {
//             if _descending {
//                 Pair::Descending
//             } else {
//                 Pair::Ascending
//             }
//         }
//         _ => default,
//     }
// }

// /// Verify that a message has been processed before the specified deadline
// pub fn check_deadline(block: &BlockInfo, deadline: Timestamp) -> Result<(), ContractError> {
//     if deadline <= block.time {
//         return Err(ContractError::DeadlinePassed);
//     }
//     Ok(())
// }

// /// Verify that the finder address is neither the sender nor the asset recipient
// pub fn validate_finder(
//     finder: &Option<Addr>,
//     sender: &Addr,
//     asset_recipient: &Option<Addr>,
// ) -> Result<(), ContractError> {
//     if finder.is_none() {
//         return Ok(());
//     }
//     if finder.as_ref().unwrap() == sender || finder == asset_recipient {
//         return Err(ContractError::InvalidInput("finder is invalid".to_string()));
//     }
//     Ok(())
// }

// pub struct SwapPrepResult {
//     pub marketplace_params: ParamsResponse,
//     pub collection_royalties: Option<RoyaltyInfoResponse>,
//     pub asset_recipient: Addr,
//     pub finder: Option<Addr>,
//     pub developer: Option<Addr>,
// }

// /// Prepare the contract for a swap transaction
// pub fn prep_for_swap(
//     deps: Deps,
//     block_info: &Option<BlockInfo>,
//     sender: &Addr,
//     collection: &Addr,
//     swap_params: &SwapParams,
// ) -> Result<SwapPrepResult, ContractError> {
//     if let Some(_block_info) = block_info {
//         check_deadline(_block_info, swap_params.deadline)?;
//     }

//     let finder = maybe_addr(deps.api, swap_params.finder.clone())?;
//     let asset_recipient = maybe_addr(deps.api, swap_params.asset_recipient.clone())?;

//     validate_finder(&finder, sender, &asset_recipient)?;

//     let config = CONFIG.load(deps.storage)?;
//     let marketplace_params = load_marketplace_params(deps, &config.marketplace_addr)?;

//     let collection_royalties = load_collection_royalties(deps, collection)?;

//     let seller_recipient = asset_recipient.unwrap_or_else(|| sender.clone());

//     Ok(SwapPrepResult {
//         marketplace_params,
//         collection_royalties,
//         asset_recipient: seller_recipient,
//         finder,
//         developer: config.developer,
//     })
// }

// /// Validate NftSwap vector token amounts, and NFT ownership
// pub fn validate_nft_swaps_for_sell(
//     deps: Deps,
//     info: &MessageInfo,
//     collection: &Addr,
//     nft_swaps: &[NftSwap],
// ) -> Result<(), ContractError> {
//     if nft_swaps.is_empty() {
//         return Err(ContractError::InvalidInput(
//             "nft swaps must not be empty".to_string(),
//         ));
//     }
//     let mut uniq_nft_token_ids: BTreeSet<String> = BTreeSet::new();
//     for (idx, nft_swap) in nft_swaps.iter().enumerate() {
//         only_nft_owner(deps, info, collection, &nft_swap.nft_token_id)?;
//         if uniq_nft_token_ids.contains(&nft_swap.nft_token_id) {
//             return Err(ContractError::InvalidInput(
//                 "found duplicate nft token id".to_string(),
//             ));
//         }
//         uniq_nft_token_ids.insert(nft_swap.nft_token_id.clone());

//         if idx == 0 {
//             continue;
//         }
//         if nft_swaps[idx - 1].token_amount < nft_swap.token_amount {
//             return Err(ContractError::InvalidInput(
//                 "nft swap token amounts must decrease monotonically".to_string(),
//             ));
//         }
//     }
//     Ok(())
// }

// /// Validate NftSwap vector token amounts, and that user has provided enough tokens
// pub fn validate_nft_swaps_for_buy(
//     info: &MessageInfo,
//     pair_nft_swaps: &Vec<PairNftSwap>,
// ) -> Result<Uint128, ContractError> {
//     if pair_nft_swaps.is_empty() {
//         return Err(ContractError::InvalidInput(
//             "pair nft swaps must not be empty".to_string(),
//         ));
//     }
//     let mut expected_amount = Uint128::zero();
//     let mut uniq_nft_token_ids: BTreeSet<String> = BTreeSet::new();

//     for pair_nft_swap in pair_nft_swaps {
//         for (idx, nft_swap) in pair_nft_swap.nft_swaps.iter().enumerate() {
//             if uniq_nft_token_ids.contains(&nft_swap.nft_token_id) {
//                 return Err(ContractError::InvalidInput(
//                     "found duplicate nft token id".to_string(),
//                 ));
//             }
//             uniq_nft_token_ids.insert(nft_swap.nft_token_id.clone());

//             expected_amount += nft_swap.token_amount;
//             if idx == 0 {
//                 continue;
//             }
//             if pair_nft_swap.nft_swaps[idx - 1].token_amount > nft_swap.token_amount {
//                 return Err(ContractError::InvalidInput(
//                     "nft swap token amounts must increase monotonically".to_string(),
//                 ));
//             }
//         }
//     }

//     let received_amount = must_pay(info, NATIVE_DENOM)?;
//     if received_amount != expected_amount {
//         return Err(ContractError::InsufficientFunds(format!(
//             "expected {} but received {}",
//             expected_amount, received_amount
//         )));
//     }
//     Ok(received_amount)
// }
