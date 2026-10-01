use std::fmt;
use serde::{Deserialize, Serialize};

mod defaults {
    pub fn size() -> u16 { 150 }
    pub fn search_field() -> String { "title".into() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderDirection {
    Asc,
    Desc,
}
impl Default for OrderDirection {
    fn default() -> Self { Self::Desc }
}

fn is_flag<'de, D>(de: D) -> Result<bool, D::Error>
where D: serde::Deserializer<'de> {
    let opt = Option::<String>::deserialize(de)?;
    match opt {
        None => Ok(false),           // not present
        Some(s) if s.is_empty() => Ok(true), // ?show_xxx
        Some(s) => match s.as_str() {
            "1" | "true" | "yes" | "y" => Ok(true),  // ?show_xxx=true
            "0" | "false" | "no" | "n" => Ok(false), // ?show_xxx=false
            _ => Err(serde::de::Error::custom("invalid bool")),
        },
    }
}

fn non_empty_string<'de, D>(de: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    if s.trim().is_empty() {
        return Err(serde::de::Error::custom("query must not be empty"));
    }
    Ok(s)
}

fn comma_separated<'de, D>(de: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    if let Some(s) = opt {
        let mut s: Vec<String> = s.split(',').map(|s| s.to_string()).collect();
        s.sort_unstable();
        s.dedup();
        Ok(s)
    } else {
        Ok(Vec::new())
    }
}

fn comma_separated_u64<'de, D>(de: D) -> Result<Vec<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    if let Some(s) = opt {
        let mut s =
        s.split(',')
            .map(|x| x.parse::<u64>().map_err(serde::de::Error::custom))
            .collect::<Result<Vec<u64>, D::Error>>()?;
        s.sort_unstable();
        s.dedup();
        Ok(s)
    } else {
        Ok(Vec::new())
    }
}

#[derive(Deserialize, Serialize, Hash)]
pub struct SearchQuery {
    // The query is the ONLY hard requirement for searching.
    #[serde(rename = "q", deserialize_with = "non_empty_string")]
    pub query: String,
    #[serde(rename = "sf", default = "defaults::search_field")]
    pub search_field: String,
    #[serde(rename = "o")]
    pub order_by: Option<String>,
    #[serde(rename = "d", default)]
    pub order_direction: OrderDirection,
    #[serde(rename = "c", default, deserialize_with = "comma_separated_u64")]
    pub categories: Vec<u64>,
    #[serde(rename = "t", default, deserialize_with = "comma_separated")]
    pub trackers: Vec<String>,
    #[serde(rename = "f", default)]
    pub from: u16,
    #[serde(rename = "s", default = "defaults::size")]
    pub size: u16,
    #[serde(rename = "xxx", default, deserialize_with = "is_flag")]
    pub show_xxx: bool,
    #[serde(rename = "unsafe", default, deserialize_with = "is_flag")]
    pub show_unsafe: bool,
    #[serde(rename = "dead", default, deserialize_with = "is_flag")]
    pub show_dead_torrents: bool,
    #[serde(rename = "seen")]
    pub seconds_since_last_seen: Option<i64>,
}
impl fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

mod browse_defaults {
    pub fn order_by() -> String { "date".into() }
}

#[derive(Deserialize, Serialize, Hash)]
pub struct BrowseQuery {
    // The query is the ONLY hard requirement for searching.
    #[serde(rename = "o", default = "browse_defaults::order_by")]
    pub order_by: String,
    #[serde(rename = "d", default)]
    pub order_direction: OrderDirection,
    #[serde(rename = "c", default, deserialize_with = "comma_separated_u64")]
    pub categories: Vec<u64>,
    #[serde(rename = "t", default, deserialize_with = "comma_separated")]
    pub trackers: Vec<String>,
    #[serde(rename = "f", default)]
    pub from: u16,
    #[serde(rename = "s", default = "defaults::size")]
    pub size: u16,
    #[serde(rename = "xxx", default, deserialize_with = "is_flag")]
    pub show_xxx: bool,
    #[serde(rename = "unsafe", default, deserialize_with = "is_flag")]
    pub show_unsafe: bool,
    #[serde(rename = "dead", default, deserialize_with = "is_flag")]
    pub show_dead_torrents: bool,
    #[serde(rename = "seen")]
    pub seconds_since_last_seen: Option<i64>,
}
impl fmt::Display for BrowseQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}