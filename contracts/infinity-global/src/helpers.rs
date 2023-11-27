use crate::{msg::QueryMsg, state::GlobalConfig};

use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdResult};

pub fn load_global_config(
    querier: &QuerierWrapper,
    infinity_global: &Addr,
) -> StdResult<GlobalConfig<Addr>> {
    querier.query_wasm_smart::<GlobalConfig<Addr>>(infinity_global, &QueryMsg::GlobalConfig {})
}

pub fn load_min_price(
    querier: &QuerierWrapper,
    infinity_global: &Addr,
    denom: &str,
) -> StdResult<Option<Coin>> {
    querier.query_wasm_smart::<Option<Coin>>(
        infinity_global,
        &QueryMsg::MinPrice {
            denom: denom.to_string(),
        },
    )
}
