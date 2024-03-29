use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Log {
    pub conf_path: String,

}

#[derive(Deserialize)]
pub struct IMap {
    pub domain: String,
    pub username: String,
    pub password: String,
    pub mailbox: String,
}

#[derive(Deserialize)]
pub struct Smtp {
    pub domain: String,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[derive(Deserialize)]
pub struct Rpc {
    pub addr: String,
    pub taskcache: String,
}

pub mod controller {
    use std::{fs, error::Error};
    use log::{debug, trace};
    use super::*;

    #[derive(Deserialize)]
    pub struct Config {
        pub imap: IMap,
        pub rpc: Rpc,
        pub log: Log,
        pub smtp: Smtp,
    }

    pub struct ConfigMgr {
        pub config: Config,
    }

    impl ConfigMgr  {
        pub fn new_with_path(path: &str) -> Result<Self, Box<dyn Error>> {
            let cfgmgr = ConfigMgr{
                config: Self::config_from_path(path)?,
            };
            
            Ok(cfgmgr)
        }

        pub fn new() -> Result<Self, Box<dyn Error>> {
            let cfgmgr = Self::new_with_path("settings.toml")?;
            
            Ok(cfgmgr)
        }

        pub fn get(&self) -> &Config {
            &self.config
        }

        fn config_from_path(path: &str) -> Result<Config, Box<dyn Error>> {
            debug!("Load config file {}", path);
            let file_content = fs::read_to_string(path)?;

            trace!("Config file {} content: {}", path, &file_content);
            let config: Config = toml::from_str(&file_content).unwrap();
            
            Ok(config)
        }
    }

}

pub mod worker {
    use std::{fs, error::Error};
    use log::{debug, trace};
    use super::*;

    #[derive(Deserialize)]
    pub struct Execute {
        pub runner: String,
    }

    #[derive(Deserialize)]
    pub struct Config {
        pub rpc: Rpc,
        pub log: Log,
        pub execute: Execute,
    }

    pub struct ConfigMgr {
        pub config: Config,
    }

    impl ConfigMgr  {
        pub fn new_with_path(path: &str) -> Result<Self, Box<dyn Error>> {
            let cfgmgr = ConfigMgr{
                config: Self::config_from_path(path)?,
            };
            
            Ok(cfgmgr)
        }

        pub fn new() -> Result<Self, Box<dyn Error>> {
            let cfgmgr = Self::new_with_path("settings.toml")?;
            
            Ok(cfgmgr)
        }

        pub fn get(&self) -> &Config {
            &self.config
        }

        fn config_from_path(path: &str) -> Result<Config, Box<dyn Error>> {
            debug!("Load config file {}", path);
            let file_content = fs::read_to_string(path)?;

            trace!("Config file {} content: {}", path, &file_content);
            let config: Config = toml::from_str(&file_content).unwrap();
            
            Ok(config)
        }
    }

}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;
    
    use super::*;

    #[test]
    fn test_read() {
        let mut file = match File::create("test-config.toml") {
            Ok(file) => file,
            Err(e) => panic!("{}", e.to_string())
        };
        
        assert!( !file.write_all(r#"
[imap]
domain = "imap.google.com"
username = "abc@gmail.com"
password = "asdfjkl;"
mailbox = "INBOX"

[smtp]
from = "Somebody <sbd@gmail.com>"
domain = "imap.google.com"
username = "abc@gmail.com"
password = "asdfjkl;"

[rpc]
addr = "[::1]:9999"
taskcache = "task.cache"

[log]
conf_path = "log4rs.yaml"
        "#.as_bytes()).is_err() );

        let cfgmgr = match controller::ConfigMgr::new_with_path("test-config.toml") {
            Ok(config) => config,
            Err(e) => panic!("{}", e)
        };

        assert_eq!(cfgmgr.get().imap.domain, "imap.google.com");
    }

}