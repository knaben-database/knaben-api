use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Hash)]
pub struct QueryBody {
    // search_type can either be score, referring to scoring based on best hit a percentage value from 0% - 100% referring to Elasticsarch's minimum_should_match
    pub search_type: Option<String>,
    // This is the search query itself
    pub query: Option<String>,
    // If set, will match the query search on the set field. The available fields are mentioned below.
    pub search_field: Option<String>,
    // The field to order by. Normally most people order by seeders or peers in descending order.
    pub order_by: Option<String>,
    // This is the direction you want to order by in. Descending, biggest number first. Ascending, Smallest number first.
    pub order_direction: Option<String>,
    // The category ids you want to search on. There are so many ids, better have a look at the RSS page, clicking on Toggle show categories to find the IDs
    pub categories: Option<Vec<u64>>,
    // This is the scroll offset. If you have a size of 10, and want to scroll to the next page, you can do from: 10, size: 10. Then from: 20, size 10 etc...
    pub from: Option<u16>,
    // The amount of results to return. Maximum size is 300
    pub size: Option<u16>,
    // To hide porn or not to hide porn.
    pub hide_xxx: Option<bool>,
    // Filters out very old results, and results that has a relatively high potential virus score.
    pub hide_unsafe: Option<bool>,
    // Relative seconds since last seen.
    pub seconds_since_last_seen: Option<i64>,
}
impl fmt::Display for QueryBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}