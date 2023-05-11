use crate::state::{BondingCurve, PoolConfig, PoolType};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct PoolInfo {
    /// The address of the NFT collection contract
    pub collection: String,
    /// The address of the owner of the pool
    pub owner: String,
    /// The address of the recipient of assets traded into the pool
    pub asset_recipient: Option<String>,
    /// The type of assets held by the pool
    pub pool_type: PoolType,
    /// The bonding curve used to calculate the spot price
    pub bonding_curve: BondingCurve,
    /// A moving value used to derive the price at which the pool will trade assets
    /// Note: this value is not necessarily the final sale price for pool assets
    pub spot_price: Uint128,
    /// The amount by which the spot price will increment/decrement
    /// For linear curves, this is the constant amount
    /// For exponential curves, this is the percentage amount (treated as basis points)
    pub delta: Uint128,
    /// The percentage of the swap that will be paid to the finder of a trade
    pub finders_fee_bps: u64,
    /// The percentage of the swap that will be paid to the pool owner
    /// Note: this only applies to Trade pools
    pub swap_fee_bps: u64,
    /// Whether or not the tokens sold into the pool will be reinvested
    pub reinvest_tokens: bool,
    /// Whether or not the NFTs sold into the pool will be reinvested
    pub reinvest_nfts: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    // The address of the marketplace contract
    pub marketplace: String,
    // The address of the infinity index contract
    pub infinity_index: String,
    /// The configuration object for the pool
    pub pool_info: PoolInfo,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Deposit NFTs into the pool
    DepositNfts {
        collection: String,
        token_ids: Vec<String>,
    },
    // /// Withdraw tokens from a pool
    // WithdrawTokens {
    //     pool_id: u64,
    //     amount: Uint128,
    //     asset_recipient: Option<String>,
    // },
    // /// Withdraw all tokens from a pool
    // WithdrawAllTokens {
    //     pool_id: u64,
    //     asset_recipient: Option<String>,
    // },
    // /// Withdraw NFTs from a pool
    // WithdrawNfts {
    //     pool_id: u64,
    //     nft_token_ids: Vec<String>,
    //     asset_recipient: Option<String>,
    // },
    // /// Update the parameters of a pool
    // UpdatePoolConfig {
    //     asset_recipient: Option<String>,
    //     delta: Option<Uint128>,
    //     spot_price: Option<Uint128>,
    //     finders_fee_bps: Option<u64>,
    //     swap_fee_bps: Option<u64>,
    //     reinvest_tokens: Option<bool>,
    //     reinvest_nfts: Option<bool>,
    // },
    // // Activate a pool so that it may begin accepting trades
    // SetIsActive {
    //     is_active: bool,
    // },
    // /// Remove a pool from contract storage and indexing
    // RemovePool {
    //     pool_id: u64,
    //     asset_recipient: Option<String>,
    // },
    // SwapNftsForTokens {
    //     token_id: String,
    //     min_output: Uint128,
    //     asset_recipient: String,
    //     finder: Option<String>,
    // },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(PoolConfigResponse)]
    PoolConfig {},
    #[returns(NftDepositsResponse)]
    NftDeposits {
        query_options: Option<QueryOptions<String>>,
    },
}

/// QueryOptions are used to paginate contract queries
#[cw_serde]
pub struct QueryOptions<T> {
    /// Whether to sort items in ascending or descending order
    pub descending: Option<bool>,
    /// The key to start the query after
    pub start_after: Option<T>,
    // The number of items that will be returned
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct PoolConfigResponse {
    pub config: PoolConfig,
}

#[cw_serde]
pub struct NftDepositsResponse {
    pub nft_deposits: Vec<String>,
}
