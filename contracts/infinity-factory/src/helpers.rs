use crate::ContractError;

use cosmwasm_std::{instantiate2_address, Addr, Binary, Deps, Env};
use sha2::{Digest, Sha256};

pub fn generate_salt(sender: &Addr, counter: u64) -> Binary {
    let mut hasher = Sha256::new();
    hasher.update(sender.as_bytes());
    hasher.update(counter.to_be_bytes());
    hasher.finalize().to_vec().into()
}

pub fn generate_instantiate_2_addr(
    deps: Deps,
    env: &Env,
    sender: &Addr,
    counter: u64,
    code_id: u64,
) -> Result<(Addr, Binary), ContractError> {
    let code_res = deps.querier.query_wasm_code_info(code_id)?;

    let salt = generate_salt(sender, counter);

    // predict the contract address
    let addr_raw = instantiate2_address(
        &code_res.checksum,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        &salt,
    )?;

    let addr = deps.api.addr_humanize(&addr_raw)?;

    Ok((addr, salt))
}
