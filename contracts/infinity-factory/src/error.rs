use cosmwasm_std::Instantiate2AddressError;
use cosmwasm_std::StdError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),
}
