use std::path::PathBuf;
use chrono::*;
use std::ops::*;
use cmd_lib::*;

pub use crate::*;
pub use crate::conf::*;
pub use crate::styles::*;

#[macro_export]
macro_rules! Config_get { ($e:expr) => {
    match $e {
        "basedir" => { CONFIG.read().unwrap().basedir.clone().unwrap() },
        //"blink" => { CONFIG.read().unwrap().blink.unwrap_or(true) },
        _ => { panic!("unknown config key") }
    }
}; }

pub fn pick_file() -> String {
    run_fun!(
        ls | fzf;
    ).unwrap_or_else(|_|
        std::process::exit(1)
    )
}

pub fn path_normalize(path_str: &str) -> String {
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

    conf_path.to_str().expect("wrong path").into()
}

pub fn get_today() -> String {
    Local::now().date_naive().to_string()
}
pub fn get_yesterday() -> String {
    Local::now().add(chrono::Duration::days(-1)).date_naive().to_string()
}
pub fn get_tomorrow() -> String {
    Local::now().add(chrono::Duration::days(1)).date_naive().to_string()
}

pub fn get_box_alias(name_in: &str) -> Option<String> {
    let alias = match name_in {
        _ if name_in == get_today() => "today",
        _ if name_in == get_tomorrow() => "tomorrow",
        _ if name_in == get_yesterday() => "yesterday",
        _ => "",
    };

    if alias.is_empty() { None }
    else { Some(alias.to_string()) }
}
pub fn get_box_unalias(alias: &str) -> String {
    match alias {
        "today" => get_today(),
        "yesterday" => get_yesterday(),
        "tomorrow" => get_tomorrow(),
        _ => alias.into(),
    }
}

pub fn get_inbox_file(inbox: &str) -> PathBuf {
    let basedir = PathBuf::from(Config_get!("basedir"));

    basedir.join(get_box_unalias(inbox)).with_extension("md")
}

pub fn confirm(question: &str) -> bool {
    inquire::Confirm::new(question)
        .with_default(false)
        .with_render_config(get_confirm_style())
        .prompt().unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aliases() {
        assert_eq!(get_box_alias(&get_today()), Some("today".into()));
        assert_eq!(get_box_alias(&get_yesterday()), Some("yesterday".into()));
        assert_eq!(get_box_alias(&get_tomorrow()), Some("tomorrow".into()));
        assert_eq!(get_box_alias("dummy"), None);
        assert_eq!(get_box_alias(""), None);
    }

    #[test]
    fn test_unaliases() {
        assert_eq!(get_box_unalias("today"), get_today());
        assert_eq!(get_box_unalias("yesterday"), get_yesterday());
        assert_eq!(get_box_unalias("tomorrow"), get_tomorrow());
        assert_eq!(get_box_unalias("dummy"), "dummy".to_string());
    }

    #[test]
    fn test_path_normalize() {
        assert_eq!(path_normalize("~/dummy"),
            dirs::home_dir().unwrap().join("dummy").to_str().unwrap());
        assert_eq!(path_normalize("dummy"),
            std::env::current_dir().unwrap().join("dummy").to_str().unwrap());
        assert_eq!(path_normalize("~dummy"),
            std::env::current_dir().unwrap().join("~dummy").to_str().unwrap());
        assert_eq!(path_normalize("./dummy"),
            std::env::current_dir().unwrap().join("dummy").to_str().unwrap());
        assert_eq!(path_normalize("/dummy"), "/dummy");
    }
}
