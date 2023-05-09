use cosmwasm_std::StdError;
use infinity_shared::InfinityError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    InfinityError(#[from] InfinityError),
}
