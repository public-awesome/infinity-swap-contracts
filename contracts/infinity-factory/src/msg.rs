use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use infinity_pool::state::BondingCurve;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateTokenPool {
        collection: String,
        asset_recipient: Option<String>,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        finders_fee_bps: u64,
    },
    CreateNftPool {
        collection: String,
        asset_recipient: Option<String>,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        finders_fee_bps: u64,
    },
    CreateTradePool {
        collection: String,
        asset_recipient: Option<String>,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        finders_fee_bps: u64,
        swap_fee_bps: u64,
        reinvest_tokens: bool,
        reinvest_nfts: bool,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    InfinityGlobal {},
}
