// This file is meant to hide and obscure the db result a bit

use std::collections::HashMap;
use std::error::Error;
use lazy_static::lazy_static;
use serde_json::{Value};

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


        replace_map.insert("Title", "title");
        replace_map.insert("InfoHash", "hash");
        replace_map.insert("CachedOrigin", "cachedOrigin");
        replace_map.insert("Tracker", "tracker");
        replace_map.insert("TrackerId", "trackerId");
        replace_map.insert("LastSeen", "lastSeen");
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
        return replace_map
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

        if let Some(source) = hit_obj.remove("_source") {
            hit_obj.clear();
            let source_obj = source.as_object().ok_or("_source is not an object")?;
            for (k, v) in source_obj {
                if let Some(new_key) = REPLACER.get(k.as_str()) {
                    hit_obj.insert(new_key.to_string(), v.clone());
                }
            }
            hit_obj.insert("id".to_string(), id);
        }
    }

    Ok(hits.to_owned())
}
