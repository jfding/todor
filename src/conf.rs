use std::path::PathBuf;
use std::sync::RwLock;
use dirs;
use toml;
use serde::Deserialize;
use lazy_static::lazy_static;

use crate::util::*;

const DEF_CONFIG_PATH : &str = ".config/todor/todor.toml";
const DATA_BASE : &str = ".local/share/todor";
const DEF_CONFIG_CONTENT: &str = r#"# config for todor in toml

## base directory for todor data
basedir = "~/.local/share/todor"

## blink the icons of items or not
blink = true
"#;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::load(None));
}

pub fn get_default_basedir() -> String {
    // for windows compatibility
    let rel_base :PathBuf = DATA_BASE.split("/").collect();
    dirs::home_dir()
        .expect("cannot get home dir")
        .join(rel_base)
        .to_str()
        .expect("cannot convert path to string")
        .to_string()
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
            basedir: Some(get_default_basedir()),
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
            confp = PathBuf::from(util::path_normalize(&path_str));
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
            conf.basedir = Some(util::path_normalize(&basedir))
        }

        work_conf.update_with(&conf);
        work_conf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_basedir() {
        assert!(get_default_basedir().contains(".local/share/todor"));
    }

    #[test]
    fn test_config_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let testtoml = temp_dir.path().join("config.toml");
        let testcontent = r#"basedir = "/tmp/.todor-test/"
        blink = false
        "#;
        std::fs::write(&testtoml, testcontent).expect("write err");
        let conf = Config::load(Some(testtoml.to_str().unwrap().into()));
        assert_eq!(conf.basedir, Some("/tmp/.todor-test/".into()));
        assert_eq!(conf.blink, Some(false));
    }

    #[test]
    fn test_config_default() {
        let conf = Config::default();
        assert_eq!(conf.basedir, Some(get_default_basedir()));
        assert_eq!(conf.blink, Some(true));
    }

    #[test]
    fn test_config_update() {
        let mut conf = Config::default();
        let aconf = Config {
            basedir: Some("/nowhere".into()),
            blink: Some(false),
        };
        conf.update_with(&aconf);
        assert_eq!(conf.basedir, Some("/nowhere".into()));
        assert_eq!(conf.blink, Some(false));
    }
}
