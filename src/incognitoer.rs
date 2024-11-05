// This file is meant to hide and obscure the db result a bit

use crate::{KNABEN_DOMAIN, PROXY_DOMAIN};
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ops::Deref;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maps() {
        for value in REPLACER.values() {
            assert!(SORTER.contains_key(value), "Key {} not found in SORTER", value);
        }
        for key in SORTER.keys() {
            let mut found = false;
            for value in REPLACER.values() {
                found |= value == key;
            }
            assert!(found, "Value {} not found in REPLACER", key);
        }
        for (key, value) in REPLACER.iter() {
            assert_eq!(
                REPLACER_REVERSE.get(value),
                Some(key),
                "Key-value pair {}-{} not found in REPLACER_REVERSE",
                value,
                key
            );
        }
        for (key, value) in REPLACER_REVERSE.iter() {
            assert_eq!(REPLACER.get(value), Some(key), "Key-value pair {}-{} not found in REPLACER", value, key);
        }
    }
}

lazy_static! {
    static ref REQUIRED_FIELDS: HashSet<&'static str> = {
        let mut required_fields: HashSet<&str> = HashSet::new();

        required_fields.insert("title");
        required_fields.insert("hash");
        required_fields.insert("cachedOrigin");
        required_fields.insert("tracker");
        required_fields.insert("trackerId");
        required_fields.insert("lastSeen");
        required_fields.insert("grabs");
        required_fields.insert("seeders");
        required_fields.insert("peers");
        required_fields.insert("categoryId");
        required_fields.insert("category");
        required_fields.insert("details");
        required_fields.insert("link");
        required_fields.insert("magnetUrl");
        required_fields.insert("date");
        required_fields.insert("bytes");
        required_fields.insert("virusDetection");
        return required_fields;
    };
}

lazy_static! {
    static ref SORTER: HashMap<&'static str, &'static str> = {
        let mut sort_map: HashMap<&str, &str> = HashMap::new();

        sort_map.insert("title", "Title.keyword");
        sort_map.insert("hash", "InfoHash.keyword");
        sort_map.insert("cachedOrigin", "CachedOrigin.keyword");
        sort_map.insert("tracker", "Tracker.keyword");
        sort_map.insert("trackerId", "TrackerId.keyword");
        sort_map.insert("lastSeen", "LastSeen");
        sort_map.insert("grabs", "Grabs");
        sort_map.insert("seeders", "Seeders");
        sort_map.insert("peers", "Peers");
        sort_map.insert("categoryId", "Category");
        sort_map.insert("category", "CategoryDesc.keyword");
        sort_map.insert("details", "Details.keyword");
        sort_map.insert("link", "Link.keyword");
        sort_map.insert("magnetUrl", "MagnetUri.keyword");
        sort_map.insert("date", "PublishDate");
        sort_map.insert("bytes", "Size");
        sort_map.insert("virusDetection", "VirusDetection");
        return sort_map;
    };
}

pub fn format_sort(sort: String) -> Result<String, Box<dyn Error>> {
    let sort = SORTER
        .get(sort.deref())
        .ok_or(format!("Unable to sort by {} because it doesn't exist in the mapping", sort))?;
    return Ok(sort.to_string());
}

lazy_static! {
    static ref REPLACER: HashMap<&'static str, &'static str> = {
        let mut replace_map: HashMap<&str, &str> = HashMap::new();

        replace_map.insert("Title", "title");
        replace_map.insert("InfoHash", "hash");
        replace_map.insert("CachedOrigin", "cachedOrigin");
        replace_map.insert("Tracker", "tracker");
        replace_map.insert("TrackerId", "trackerId");
        replace_map.insert("LastSeen", "lastSeen");
        replace_map.insert("Grabs", "grabs");
        replace_map.insert("Seeders", "seeders");
        replace_map.insert("Peers", "peers");
        replace_map.insert("Category", "categoryId");
        replace_map.insert("CategoryDesc", "category");
        replace_map.insert("Details", "details");
        replace_map.insert("Link", "link");
        replace_map.insert("MagnetUri", "magnetUrl");
        replace_map.insert("PublishDate", "date");
        replace_map.insert("Size", "bytes");
        replace_map.insert("VirusDetection", "virusDetection");
        return replace_map;
    };
}

pub fn format_field(field: &String) -> Result<String, Box<dyn Error>> {
    let field = REPLACER_REVERSE.get(field.as_str()).ok_or(format!("Unable to find {}", field))?;
    return Ok(field.to_string());
}

lazy_static! {
    static ref REPLACER_REVERSE: HashMap<&'static str, &'static str> = {
        let mut replace_r_map: HashMap<&str, &str> = HashMap::new();

        replace_r_map.insert("title", "Title");
        replace_r_map.insert("hash", "InfoHash");
        replace_r_map.insert("cachedOrigin", "CachedOrigin");
        replace_r_map.insert("tracker", "Tracker");
        replace_r_map.insert("trackerId", "TrackerId");
        replace_r_map.insert("lastSeen", "LastSeen");
        replace_r_map.insert("grabs", "Grabs");
        replace_r_map.insert("seeders", "Seeders");
        replace_r_map.insert("peers", "Peers");
        replace_r_map.insert("categoryId", "Category");
        replace_r_map.insert("category", "CategoryDesc");
        replace_r_map.insert("details", "Details");
        replace_r_map.insert("link", "Link");
        replace_r_map.insert("magnetUrl", "MagnetUri");
        replace_r_map.insert("date", "PublishDate");
        replace_r_map.insert("bytes", "Size");
        replace_r_map.insert("virusDetection", "VirusDetection");
        return replace_r_map;
    };
}

pub fn obscure(mut val: Value) -> Result<Value, Box<dyn Error>> {
    let obj = val.as_object_mut().ok_or("Root is not an object")?;
    let hits = obj.get_mut("hits").ok_or("No 'hits' field in root")?;
    let hits_obj = hits.as_object_mut().ok_or("'hits' is not an object")?;
    let hits_arr = hits_obj.get_mut("hits").ok_or("'hits' field in 'hits' object is missing")?;
    let arr = hits_arr.as_array_mut().ok_or("'hits' is not an array")?;

    for hit in arr {
        let hit_obj = hit.as_object_mut().ok_or("Hit is not an object")?;

        let id = hit_obj.remove("_id").ok_or("Source doesn't contain an id")?;
        let score = hit_obj.remove("_score").ok_or("Source doesn't contain an score")?;

        if let Some(source) = hit_obj.remove("_source") {
            hit_obj.clear();
            let source_obj = source.as_object().ok_or("_source is not an object")?;
            for (k, v) in source_obj {
                if let Some(new_key) = REPLACER.get(k.as_str()) {
                    hit_obj.insert(new_key.to_string(), v.clone());
                }
            }

            if let Some(v) = hit_obj.get("link") {
                if let Some(link) = v.as_str() {
                    let mut link = link.to_string();
                    link = link.replace("http://127.0.0.1:9117/", &format!("https://{}/live/", KNABEN_DOMAIN));
                    link = link.replace("***REMOVED***", "***REMOVED***");
                    link = link.replace("jackett_apikey=***REMOVED***&", "");
                    if let Some(details) = hit_obj.get("details") {
                        link = format!("{}&knabensource={}", link, urlencoding::encode(details.as_str().unwrap()));
                    }
                    hit_obj["link"] = Value::String(link);
                }
            }

            if let Some(id) = hit_obj.get("trackerId").and_then(|value| value.as_str()) {
                if id == "thepiratebay" {
                    if let Some(Value::String(details)) = hit_obj.get("details") {
                        if details.contains("://") {
                            let beginstring = details.split("://").collect::<Vec<&str>>()[1];
                            let endstring = beginstring.split("/").collect::<Vec<&str>>()[0];
                            let new_details = details.replace(endstring, &format!("{}/thepiratebay", PROXY_DOMAIN));
                            hit_obj.insert("details".to_string(), Value::String(new_details));
                        }
                    }
                    if let Some(Value::String(title)) = hit_obj.get("title") {
                        if title.len() == 64 {
                            let new_title = format!("{} ...", title);
                            hit_obj.insert("title".to_string(), Value::String(new_title));
                        }
                    }
                }
            }

            // Ensure all fields are there
            for v in REQUIRED_FIELDS.iter() {
                if hit_obj.get(v.to_owned()).is_none() {
                    hit_obj.insert(v.to_string(), Value::Null);
                }
            }

            hit_obj.insert("score".to_string(), score);
            hit_obj.insert("id".to_string(), id);
        }
    }

    Ok(hits.to_owned())
}
