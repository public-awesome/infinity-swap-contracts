use crate::ContractError;
use anyhow::Error;
use cw_multi_test::AppResponse;

pub fn assert_error(res: Result<AppResponse, Error>, expected: ContractError) {
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        expected.to_string()
    );
}
