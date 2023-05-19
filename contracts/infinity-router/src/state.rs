use crate::msg::NftOrder;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Deque, Item};

// The address of the infinity global contract
pub const INFINITY_GLOBAL: Item<Addr> = Item::new("ig");

pub const NFT_ORDERS: Deque<NftOrder> = Deque::new("nft_orders");

#[cw_serde]
pub struct SwapContext {
    pub collection: Addr,
    pub original_sender: Addr,
    pub robust: bool,
    pub asset_recipient: Option<Addr>,
    pub finder: Option<Addr>,
    pub balance: Uint128,
}

pub const SWAP_CONTEXT: Item<SwapContext> = Item::new("swap_context");
