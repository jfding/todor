use std::path::PathBuf;
use dirs;
use toml;
use serde::Deserialize;

use crate::util::*;

const DEF_CONFIG: &str = r#"# config for todor in toml

## base directory for todor data, if not set, use default as below
#basedir = "~/.local/share/todor"
"#;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    /// base directory for todor data
    pub basedir: Option<String>,
}

impl Config {
    pub fn load(path_str: Option<String>) -> Self {
        let confp;
        if let Some(path_str) = path_str {
            confp = util::path_normalize(path_str);
            if !confp.exists() {
                eprintln!("specified config file not found, ignore it");
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

                std::fs::write(confp.clone(), DEF_CONFIG)
                    .expect("cannot create config file");
            }
        }

        let mut conf :Config = toml::from_str(
                 &std::fs::read_to_string(&confp)
                .expect("cannot read config file"))
            .expect("cannot parse config file");

        if let Some(basedir) = conf.basedir {
            conf.basedir = Some(util::path_normalize(basedir)
                                .to_str()
                                .unwrap()
                                .to_string()
                               )
        }

        conf
    }
}
