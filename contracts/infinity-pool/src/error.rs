use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid pool: {0}")]
    InvalidPool(String),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("Unable to remove pool: {0}")]
    UnableToRemovePool(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Deadline passed")]
    DeadlinePassed,

    #[error("Swap error: {0}")]
    SwapError(String),
}
