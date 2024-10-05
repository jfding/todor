use std::path::PathBuf;
use chrono::*;
use std::ops::*;
use cmd_lib::*;
use colored::Colorize;
use crossterm::execute;
use crossterm::cursor::SetCursorStyle::*;

pub use crate::*;
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

pub fn match_routine(kind: &str, s_date_str: &str) -> bool {
    let mut s_date = NaiveDate::parse_from_str(s_date_str, "%Y-%m-%d").unwrap();
    if kind == "m" {
        while s_date < Local::now().date_naive() {
            s_date = s_date + chrono::Months::new(1);
        }
    } else {
        let steps = match kind {
            "d" => 1,
            "w" => 7,
            "b" => 14,
            "q" => 28,
            _ => panic!("unknown routine kind"),
        };
        while s_date < Local::now().date_naive() {
            s_date += chrono::Duration::days(steps);
        }
    }

    s_date == Local::now().date_naive()
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
        "inbox" => taskbox::INBOX_NAME.into(),
        "routine" | "routines" => taskbox::ROUTINE_BOXNAME.into(),
        _ => alias.into(),
    }
}

pub fn get_inbox_file(inbox: &str) -> PathBuf {
    let basedir = PathBuf::from(Config_get!("basedir"));
    basedir.join(get_box_unalias(inbox)).with_extension("md")
}

pub fn i_confirm(question: &str) -> bool {
    inquire::Confirm::new(question)
        .with_default(false)
        .with_render_config(get_confirm_style())
        .prompt().unwrap_or(false)
}

pub fn i_gettext() -> String {
    execute!(std::io::stdout(), BlinkingBlock).expect("failed to set cursor");
    let input = inquire::Text::new("")
            .with_render_config(get_text_input_style())
            .with_help_message("<enter> | ctrl+c")
            .with_placeholder("something to do?")
            .prompt().unwrap_or_else(|_| String::new());
    execute!(std::io::stdout(), DefaultUserShape).expect("failed to set cursor");
    input.trim().to_string()
}

pub fn i_select(tasks: Vec<String>, title: &str) -> Vec<String> {
    execute!(std::io::stdout(), BlinkingBlock).expect("failed to set cursor");
    let mut selected = inquire::MultiSelect::new(title, tasks)
        .with_render_config(get_multi_select_style())
        .with_vim_mode(true)
        .with_page_size(10)
        .with_help_message("h/j/k/l | ←↑↓→ | <space> | <enter> | ctrl+c")
        .prompt().unwrap_or_else(|_| std::process::exit(1));
    execute!(std::io::stdout(), DefaultUserShape).expect("failed to set cursor");
    selected.retain(|x| !x.contains(WARN));
    selected
}

pub fn i_getdate(routine_kind: &str) -> String {
    inquire::DateSelect::new(&format!(" {} from:",S_routine!(routine_kind)))
        .with_render_config(get_date_input_style())
        .with_help_message("h/j/k/l | <enter> | ctrl+c")
        .prompt().unwrap_or_else(|_| {
                println!("{}", S_empty!("No starting date selected, skip."));
                std::process::exit(1)
            }).to_string()
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
