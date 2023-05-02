use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Invalid ask: {0}")]
    InvalidInput(String),

    #[error("Match error: {0}")]
    MatchError(String),

    #[error("Price mismatch: {0}")]
    PriceMismatch(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),
}
