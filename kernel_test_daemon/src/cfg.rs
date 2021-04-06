use serde_derive::Deserialize;
use std::{fs, error::Error};

use log::{error, warn, info, debug, trace};

#[derive(Deserialize)]
pub struct Config {
    pub imap: IMap,
    pub rpc: Rpc,
    pub log: Log,
}

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
pub struct Rpc {
    pub addr: String,
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
        "#.as_bytes()).is_err() );

        let cfgmgr = match ConfigMgr::new_with_path("test-config.toml") {
            Ok(config) => config,
            Err(e) => panic!("{}", e.to_string())
        };

        assert_eq!(cfgmgr.get().imap.domain, "imap.google.com");
    }

}