use cosmwasm_std::{StdError, Uint128};
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

    #[error("Insufficient funds: expected {expected}, received {received}")]
    InsufficientFunds {
        expected: Uint128,
        received: Uint128,
    },
}
