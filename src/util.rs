use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use chrono::*;
use std::ops::*;
use cmd_lib::*;
use colored::Colorize;

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

pub fn glance_all() {
    if cfg!(windows) {
        println!("Sorry, this feature is not supported on Windows.");
        return;
    }

    let wildpat = format!("{}/*.md", Config_get!("basedir"));
    let pager = "bat --paging=always -l md";
    let pager_fallback = "less";

    run_cmd!(
      sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager 2>/dev/null"
    ).unwrap_or_else(|_|
        run_cmd!(
          sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager_fallback"
        ).unwrap_or_else(|_| std::process::exit(1)));
}

pub fn edit_box(inbox_path: &Path, diffwith: Option<String>) {
    if let Some(other) = diffwith {
        let otherf = if other.ends_with(".md") {
            &other
        } else {
            &format!("{}/{}.md", Config_get!("basedir"), get_box_unalias(&other))
        };

        println!("editing : {} v.s. {}", S_fpath!(inbox_path.display()), S_fpath!(otherf));
        if run_cmd!(
            vimdiff $inbox_path $otherf 2>/dev/null
        ).is_err() {
            println!("cannot launch vimdiff, ignore --diffwith option");
        }

    } else {
        let editor = env::var("EDITOR").unwrap_or_else(|_|
                                        if cfg!(windows) {
                                            "notepad".into()
                                        } else {
                                            "vi".into()
                                        });

        let nulldev = if cfg!(windows) {
                          "NUL"
                      } else {
                          "/dev/null"
                      };

        println!("editing : {}", S_fpath!(inbox_path.display()));
        run_cmd!(
            $editor $inbox_path 2> $nulldev
        ).expect("cannot launch $EDITOR or default editor")
    }
}

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

pub fn get_inbox_file(inbox: Option<String>) -> PathBuf {
    let basedir = PathBuf::from(Config_get!("basedir"));
    fs::create_dir_all(&basedir).expect("Failed to create base directory");

    basedir
        .join(get_box_unalias(&inbox.unwrap_or(INBOX_NAME.into())))
        .with_extension("md")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_basedir() {
        assert!(get_default_basedir().contains(".local/share/todor"));
    }

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
