use std::env;
use std::fmt::Display;
use std::net::Ipv4Addr;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONFIG: Config = {
        let config = Config::from_env();
        println!("Loaded config {}", config);
        config
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn test_all_config_fields_exists() {
        let config = Config::from_env();
        let config_string = config.to_string();
        let config_debug = format!("{:?}", config).trim_start_matches("Config {").trim_end_matches("}").to_string();
        
        // Collecting all the keys for config display implementation
        let mut config_display_keys = HashSet::new();
        for line in config_string.lines() {
            if line.is_empty() {
                continue;
            }
            let key_value = line.split("=").collect::<Vec<&str>>().first().unwrap().trim().to_lowercase();
            config_display_keys.insert(key_value);
        }
        
        // Collecting all the keys for the actual config
        let mut config_keys = HashSet::new();
        for split in config_debug.split(", ").collect::<Vec<&str>>().iter() {
            if split.is_empty() {
                continue;
            }
            let key_value = split.split(":").collect::<Vec<&str>>().first().unwrap().trim().to_lowercase();
            config_keys.insert(key_value);
        }
        
        for key in config_display_keys.iter() {
            assert!(config_keys.contains(key), "Config display key: \"{}\" not found in config struct", key);
        }
        for key in config_keys.iter() {
            assert!(config_display_keys.contains(key), "Config element: \"{}\" not found in config display", key);
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub addr: Ipv4Addr,
    pub port: u16,
    pub index: String,
    pub knaben_domain: String,
    pub proxy_domain: String,
    pub elasticsearch_host: String,  // Host or ip address
    pub elasticsearch_port: u16,
    pub redis_host: String,
    pub redis_port: u16,
    // For each request, it takes a certain time to process the request.
    // What we can do to rate limit fairly, is to check the time it took, and to multiply it by this value.
    // The user will only be able to send a request once per time_took x rate_limiting_multiplier
    //pub rate_limiting_multiplier: Option<f32>
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: Ipv4Addr::new(127, 0, 0, 1),
            port: 8001,
            index: "torrent_cache3".to_string(),
            knaben_domain: "knaben.org".to_string(),
            proxy_domain: "knaben.xyz".to_string(),
            elasticsearch_host: "127.0.0.1".to_string(),
            elasticsearch_port: 9200,
            redis_host: "127.0.0.1".to_string(),
            redis_port: 6379,
            //rate_limiting_multiplier: None,
        }
    }
}
impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!("\n\n  ADDR={}\n  PORT={}\n  KNABEN_DOMAIN={}\n  PROXY_DOMAIN={}\n  ELASTICSEARCH_HOST={}\n  ELASTICSEARCH_PORT={}\n  INDEX={}\n  REDIS_HOST={}\n  REDIS_PORT={}\n", self.addr, self.port, self.knaben_domain, self.proxy_domain, self.elasticsearch_host, self.elasticsearch_port, self.index, self.redis_host, self.redis_port);
        // if let Some(rlm) = &self.rate_limiting_multiplier {
        //     str += "  RATE_LIMITING=enabled\n";
        //     str += &format!("  RATE_LIMITING_MULTIPLIER={}\n", rlm);
        // }
        write!(f, "{}", str)
    }   
}

impl Config {
    fn from_env() -> Self {
        dotenv::dotenv().ok();

        let addr = env::var("ADDR")
            .map(|s| s.parse().expect("Failed to parse ADDR environment variable into an IPv4 Address"))
            .unwrap_or(Self::default().addr);

        let port = env::var("PORT")
            .map(|s| s.parse().expect("Failed to parse PORT environment variable into an unsigned 16 integer"))
            .unwrap_or(Self::default().port);

        let index = env::var("INDEX").unwrap_or(Self::default().index);

        let knaben_domain = env::var("KNABEN_DOMAIN").unwrap_or(Self::default().knaben_domain);

        let proxy_domain = env::var("PROXY_DOMAIN").unwrap_or(Self::default().proxy_domain);

        let elasticsearch_host = env::var("ELASTICSEARCH_HOST").unwrap_or(Self::default().elasticsearch_host);

        let elasticsearch_port = env::var("ELASTICSEARCH_PORT")
            .map(|s| s.parse().expect("Failed to parse ELASTICSEARCH_PORT environment variable into an unsigned 16 integer"))
            .unwrap_or(Self::default().elasticsearch_port);

        let redis_port = env::var("REDIS_PORT")
            .map(|s| s.parse().expect("Failed to parse REDIS_PORT environment variable into an unsigned 16 integer"))
            .unwrap_or(Self::default().redis_port);

        let redis_host = env::var("REDIS_HOST").unwrap_or(Self::default().redis_host);

        // let rate_limiting_multiplier = env::var("RATE_LIMITING_MULTIPLIER")
        //     .map(|s| s.parse().expect("Failed to parse RATE_LIMITING environment variable into a float number"))
        //     .ok();

        Self {
            addr,
            port,
            index,
            knaben_domain,
            proxy_domain,
            elasticsearch_host,
            elasticsearch_port,
            redis_host,
            redis_port,
            //rate_limiting_multiplier
        }
    }
}