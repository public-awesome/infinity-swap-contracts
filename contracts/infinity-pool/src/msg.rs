use crate::state::{BondingCurve, PoolType};

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
    // The address of the global gov contract
    pub global_gov: String,
    // The address of the infinity index contract
    pub infinity_index: String,
    /// The configuration object for the pool
    pub pool_info: PoolInfo,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetIsActive {
        is_active: bool,
    },
    /// Swap NFTs for tokens
    SwapNftsForTokens {
        token_id: String,
        min_output: Uint128,
        asset_recipient: String,
        finder: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
