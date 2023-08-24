use crate::ContractError;

use cosmwasm_std::{instantiate2_address, Addr, Binary, Deps, Env, Order};
use sg_index_query::{QueryBound, QueryOptions, QueryOptionsInternal};
use sha2::{Digest, Sha256};
use std::cmp::{max, min};

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

pub fn index_range_from_query_options(
    num_pairs: u64,
    query_options: QueryOptions<u64>,
) -> Box<dyn Iterator<Item = u64>> {
    let QueryOptionsInternal {
        limit,
        order,
        ..
    } = query_options.unpack(&(|&offset| offset), None, None);

    let limit = limit as u64;

    let qo_min = query_options.min.unwrap_or(QueryBound::Inclusive(0u64));
    let min_index = match qo_min {
        QueryBound::Inclusive(min_index) => min_index,
        QueryBound::Exclusive(min_index) => min_index + 1,
    };

    let qo_max = query_options.max.unwrap_or(QueryBound::Inclusive(u64::MAX));
    let max_index = min(
        match qo_max {
            QueryBound::Inclusive(max_index) => max_index,
            QueryBound::Exclusive(max_index) => max_index - 1,
        },
        num_pairs - 1,
    );

    let range: Box<dyn Iterator<Item = u64>> = match order {
        Order::Ascending => {
            Box::new(min_index..=min(min_index.saturating_add(limit - 1), max_index))
        },
        Order::Descending => {
            Box::new((max(max_index.saturating_sub(limit - 1), min_index)..=max_index).rev())
        },
    };

    range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_index_range_from_query_options() {
        let result = index_range_from_query_options(
            10,
            QueryOptions {
                descending: None,
                limit: None,
                min: None,
                max: None,
            },
        );
        assert_eq!(result.collect::<Vec<u64>>(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let result = index_range_from_query_options(
            10,
            QueryOptions {
                descending: Some(true),
                limit: None,
                min: None,
                max: None,
            },
        );
        assert_eq!(result.collect::<Vec<u64>>(), vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);

        let result = index_range_from_query_options(
            10,
            QueryOptions {
                descending: None,
                limit: Some(5),
                min: None,
                max: None,
            },
        );
        assert_eq!(result.collect::<Vec<u64>>(), vec![0, 1, 2, 3, 4]);

        let result = index_range_from_query_options(
            10,
            QueryOptions {
                descending: None,
                limit: None,
                min: Some(QueryBound::Inclusive(5)),
                max: None,
            },
        );
        assert_eq!(result.collect::<Vec<u64>>(), vec![5, 6, 7, 8, 9]);

        let result = index_range_from_query_options(
            10,
            QueryOptions {
                descending: None,
                limit: None,
                min: None,
                max: Some(QueryBound::Inclusive(5)),
            },
        );
        assert_eq!(result.collect::<Vec<u64>>(), vec![0, 1, 2, 3, 4, 5]);

        let result = index_range_from_query_options(
            10,
            QueryOptions {
                descending: Some(true),
                limit: Some(6),
                min: Some(QueryBound::Exclusive(1)),
                max: Some(QueryBound::Exclusive(9)),
            },
        );
        assert_eq!(result.collect::<Vec<u64>>(), vec![8, 7, 6, 5, 4, 3]);
    }
}
