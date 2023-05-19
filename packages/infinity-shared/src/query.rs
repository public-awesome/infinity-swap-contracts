use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order;
use cw_storage_plus::{Bound, PrimaryKey};

// Query limits
pub const DEFAULT_QUERY_LIMIT: u32 = 10;
pub const MAX_QUERY_LIMIT: u32 = 100;

/// QueryOptions are used to paginate contract queries
#[cw_serde]
pub struct QueryOptions<T> {
    /// Whether to sort items in ascending or descending order
    pub descending: Option<bool>,
    /// The key to start the query after
    pub start_after: Option<T>,
    // The number of items that will be returned
    pub limit: Option<u32>,
}

pub fn unpack_query_options<'a, T: PrimaryKey<'a>, U, F>(
    query_options: Option<QueryOptions<U>>,
    start_after_fn: F,
) -> (usize, Order, Option<Bound<'a, T>>, Option<Bound<'a, T>>)
where
    F: Fn(U) -> Bound<'a, T>,
{
    if query_options.is_none() {
        return (DEFAULT_QUERY_LIMIT as usize, Order::Ascending, None, None);
    }
    let query_options = query_options.unwrap();

    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let mut order = Order::Ascending;
    if let Some(descending) = query_options.descending {
        if descending {
            order = Order::Descending;
        }
    };

    let (mut min, mut max) = (None, None);
    let mut bound = None;
    if let Some(start_after) = query_options.start_after {
        bound = Some(start_after_fn(start_after));
    };
    match order {
        Order::Ascending => min = bound,
        Order::Descending => max = bound,
    };

    (limit, order, min, max)
}
