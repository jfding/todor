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
            basedir: Some("~/.local/share/todor".into()),
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
        let confp;
        if let Some(path_str) = path_str {
            confp = util::path_normalize(path_str);
            if !confp.exists() {
                eprintln!("config file not found, ignore and use defaults");
                return Config::default();
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

        let conf :Config = toml::from_str(
                 &std::fs::read_to_string(&confp)
                .expect("cannot read config file"))
            .expect("cannot parse config file");

        let mut work_conf = Config::default();
        work_conf.update_with(&conf);

        if let Some(basedir) = work_conf.basedir {
            work_conf.basedir = Some(util::path_normalize(basedir)
                                .to_str()
                                .unwrap()
                                .to_string()
                               )
        }

        work_conf
    }
}
