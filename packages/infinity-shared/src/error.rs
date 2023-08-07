use cosmwasm_std::Coin;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum InfinityError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("InvalidInput: {0}")]
    InvalidInput(String),

    #[error("InsufficientFunds: expected {expected}")]
    InsufficientFunds {
        expected: Coin,
    },

    #[error("InternalError: {0}")]
    InternalError(String),
}
