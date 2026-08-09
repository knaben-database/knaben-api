use std::env;
use std::fmt::Display;
use std::net::Ipv4Addr;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONFIG: Config = {
        let config = Config::from_env().unwrap_or_else(|e| {
            println!("Failed to load config from environment: {}", e);
            println!("Using default config");
            Config::default()
        });
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
        let config = Config::from_env().unwrap_or_default();
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
        }
    }
}
impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\n  ADDR={}\n  PORT={}\n  KNABEN_DOMAIN={}\n  PROXY_DOMAIN={}\n  ELASTICSEARCH_HOST={}\n  ELASTICSEARCH_PORT={}\n  INDEX={}\n  REDIS_HOST={}\n  REDIS_PORT={}\n", self.addr, self.port, self.knaben_domain, self.proxy_domain, self.elasticsearch_host, self.elasticsearch_port, self.index, self.redis_host, self.redis_port)
    }   
}

impl Config {
    fn from_env() -> Result<Self, std::net::AddrParseError> {
        dotenv::dotenv().ok();

        let addr = env::var("ADDR")
            .map(|s| s.parse())
            .unwrap_or(Ok(Self::default().addr))?;

        let port = env::var("PORT")
            .map(|s| s.parse().unwrap_or(Self::default().port))
            .unwrap_or(Self::default().port);

        let elasticsearch_port = env::var("ELASTICSEARCH_PORT")
            .map(|s| s.parse().unwrap_or(Self::default().elasticsearch_port))
            .unwrap_or(Self::default().elasticsearch_port);

        Ok(Self {
            addr,
            port,
            index: env::var("INDEX").unwrap_or(Self::default().index),
            knaben_domain: env::var("KNABEN_DOMAIN").unwrap_or(Self::default().knaben_domain),
            proxy_domain: env::var("PROXY_DOMAIN").unwrap_or(Self::default().proxy_domain),
            elasticsearch_host: env::var("ELASTICSEARCH_HOST").unwrap_or(Self::default().elasticsearch_host),
            elasticsearch_port,
            redis_host: env::var("REDIS_HOST").unwrap_or(Self::default().redis_host),
            redis_port: env::var("REDIS_PORT")
                .map(|s| s.parse().unwrap_or(Self::default().redis_port))
                .unwrap_or(Self::default().redis_port),
        })
    }
}