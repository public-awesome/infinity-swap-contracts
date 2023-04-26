use cosmwasm_std::{Addr, Deps, StdResult};
use sg_marketplace::{
    msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg},
    state::SudoParams,
};

/// Load the marketplace params
pub fn load_marketplace_params(deps: Deps, marketplace_addr: &Addr) -> StdResult<SudoParams> {
    let marketplace_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(marketplace_addr, &MarketplaceQueryMsg::Params {})?;
    Ok(marketplace_params.params)
}
