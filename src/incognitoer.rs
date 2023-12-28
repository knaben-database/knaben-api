// This file is meant to hide and obscure the db result a bit

use std::collections::HashMap;
use std::error::Error;
use lazy_static::lazy_static;
use serde_json::{Map, Value};

lazy_static! {
    static ref DELETER: Vec<&'static str> = {
        let mut delete_vec : Vec<&str> = Vec::new();

        delete_vec.push("_index");
        delete_vec.push("_ignored");
        delete_vec.push("_type");
        return delete_vec
    };
}

lazy_static! {
    static ref REPLACER: HashMap<&'static str, &'static str> = {
        let mut replace_map : HashMap<&str, &str> = HashMap::new();

        replace_map.insert("CachedOrigin", "cachedOrigin");
        replace_map.insert("Category", "categoryId");
        replace_map.insert("CategoryDesc", "category");
        replace_map.insert("Details", "details");
        replace_map.insert("InfoHash", "hash");
        replace_map.insert("LastSeen", "lastSeen");
        replace_map.insert("Link", "link");
        replace_map.insert("MagnetUri", "magnetUrl");
        replace_map.insert("Peers", "peers");
        replace_map.insert("PublishDate", "date");
        replace_map.insert("Seeders", "seeders");
        replace_map.insert("Size", "bytes");
        replace_map.insert("Title", "title");
        replace_map.insert("Tracker", "tracker");
        replace_map.insert("TrackerId", "trackerId");
        replace_map.insert("VirusDetection", "virusDetection");
        return replace_map
    };
}

pub fn obscure(mut val: Value) -> Result<Vec<Value>, Box<dyn Error>> {
    let documents = val["hits"]["hits"].as_array_mut().ok_or("Expected hits to be an array")?.to_owned();

    let mut sources : Vec<Value> = vec![];
    for doc in documents {
        let old_source = doc.get("_source").ok_or("One or more results doesn't contain \"_source\"")?;
        let mut new_source: Map<String, Value> = Default::default();
        new_source.insert("id".to_string(), doc["_id"].clone());
        REPLACER.iter().for_each(|(key, value)| {
            if let Some(old_value) = old_source.get(key) {
                new_source.insert(value.to_string(), old_value.to_owned());
            }
        });
        sources.push(Value::Object(new_source));
    }

    Ok(sources)
}
