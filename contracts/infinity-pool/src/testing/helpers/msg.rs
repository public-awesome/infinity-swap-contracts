use cosmwasm_std::Addr;

use crate::{
    state::{Pool, PoolType},
    swap_processor::Swap,
};

#[derive(Debug)]
pub struct SwapPoolResult {
    pub user1: Addr,
    pub user2: Addr,
    pub creator: Addr,
    pub minter: Addr,
    pub collection: Addr,
    pub infinity_pool: Addr,
    pub pool: Pool,
}

pub struct SwapPoolSetup {
    pub pool_type: PoolType,
    pub spot_price: u128,
    pub finders_fee_bps: Option<u64>,
}

pub struct VendingTemplateSetup<'a> {
    pub minter: &'a Addr,
    pub collection: &'a Addr,
    pub creator: Addr,
    pub user1: Addr,
    pub user2: Addr,
}

pub struct ProcessSwapPoolResultsResponse {
    pub minter: Addr,
    pub collection: Addr,
    pub infinity_pool: Addr,
    pub pool: Pool,
    pub creator: Addr,
    pub user1: Addr,
    pub user2: Addr,
    pub token_ids: Vec<u32>,
}

pub struct NftSaleCheckParams {
    pub expected_spot_price: u128,
    pub expected_royalty_price: u128,
    pub expected_network_fee: u128,
    pub expected_finders_fee: u128,
    pub swaps: Vec<Swap>,
    pub creator: Addr,
    pub expected_seller: Addr,
    pub token_id: String,
    pub expected_nft_payer: Addr,
    pub expected_finder: Addr,
}
