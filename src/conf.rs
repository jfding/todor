use std::path::PathBuf;
use dirs;
use toml;
use serde::Deserialize;

use crate::util::*;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub basedir: Option<String>,
}

impl Config {
    fn conf_path_normalize(path_str: String) -> PathBuf {
        let conf_path;
        if path_str.starts_with("~/") {
            conf_path = PathBuf::from(path_str
                    .replace("~", dirs::home_dir()
                        .expect("cannot get home dir")
                        .to_str()
                        .unwrap()));

        } else if path_str.starts_with("./") {
            conf_path = std::env::current_dir()
                    .expect("cannot get current dir")
                    .join(path_str.to_string().strip_prefix("./").unwrap());
        } else if !path_str.starts_with("/") {
            conf_path = std::env::current_dir()
                    .expect("cannot get current dir")
                    .join(path_str);
        } else {
            conf_path = PathBuf::from(path_str);
        }
        conf_path
    }

    fn check_and_touch(conf_path: &PathBuf) {
        if !conf_path.exists() {
            std::fs::create_dir_all(conf_path.parent().unwrap())
                .expect("Failed to create base directory");

            std::fs::write(conf_path, "# config for todor in toml\n")
                .expect("cannot create config file");
        }
    }

    fn values_normalize(&mut self) {
        if let Some(basedir) = &self.basedir {
            if basedir.starts_with("~/") {
                self.basedir = Some(basedir.replace("~", dirs::home_dir()
                        .expect("cannot get home dir")
                        .to_str()
                        .unwrap()));
            }
        }
    }

    pub fn load(path_str: Option<String>) -> Self {
        let confp;
        if let Some(path_str) = path_str {
            confp = Config::conf_path_normalize(path_str);
            if !confp.exists() {
                eprintln!("specified config file not found, ignore it");
                return Config::default();
            }
        } else {
            let rel_base :PathBuf = DEF_CONFIG.split("/").collect();
            confp = dirs::home_dir()
                .expect("cannot get home dir")
                .join(rel_base);

            Config::check_and_touch(&confp);
        }


        let mut conf :Config = toml::from_str(
                 &std::fs::read_to_string(&confp)
                .expect("cannot read config file"))
            .expect("cannot parse config file");

        conf.values_normalize();
        conf
    }
}
