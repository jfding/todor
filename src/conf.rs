use std::path::PathBuf;
use std::sync::RwLock;
use dirs;
use toml;
use serde::Deserialize;
use lazy_static::lazy_static;

use crate::util::*;

const DEF_CONFIG_CONTENT: &str = r#"# config for todor in toml

## base directory for todor data
basedir = "~/.local/share/todor"

## blink the icons of items or not
blink = true
"#;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::load(None));
}

#[derive(Deserialize, Debug)]
pub struct Config {
    /// base directory for todor data
    pub basedir: Option<String>,

    /// blink the icons of items or not
    pub blink: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            basedir: Some(util::get_default_basedir()),
            blink: Some(true),
        }
    }
}

impl Config {
    pub fn update_with(&mut self, aconf: &Config) {
        if let Some(basedir) = &aconf.basedir {
            self.basedir = Some(basedir.clone());
        }

        if let Some(blink) = aconf.blink {
            self.blink = Some(blink);
        }
    }

    pub fn load(path_str: Option<String>) -> Self {
        let mut work_conf = Config::default();

        let confp;
        if let Some(path_str) = path_str {
            confp = PathBuf::from(util::path_normalize(path_str));
            if !confp.exists() {
                eprintln!("config file not found, ignore and use defaults");
                return work_conf;
            }
        } else {
            let rel_base :PathBuf = DEF_CONFIG_PATH.split("/").collect();
            confp = dirs::home_dir()
                .expect("cannot get home dir")
                .join(rel_base);

            if !confp.exists() {
                std::fs::create_dir_all(confp.parent().unwrap())
                    .expect("Failed to create base directory");

                std::fs::write(confp.clone(), DEF_CONFIG_CONTENT)
                    .expect("cannot create config file");
            }
        }

        let mut conf :Config = toml::from_str(
                 &std::fs::read_to_string(&confp)
                .expect("cannot read config file"))
            .expect("cannot parse config file");
        if let Some(basedir) = conf.basedir {
            conf.basedir = Some(util::path_normalize(basedir))
        }

        work_conf.update_with(&conf);
        work_conf
    }
}
