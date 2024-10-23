use std::path::{Path, PathBuf};
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

trait PathExt {
    fn normalize(&self) -> Option<String>;
}
impl PathExt for Path {
    fn normalize(&self) -> Option<String> {
        let real_path;

        if self.starts_with("~/") {
            real_path = dirs::home_dir()?.join(self.strip_prefix("~/").unwrap());
        } else if self.is_relative() {
            real_path = std::env::current_dir().expect("cannot get cwd?")
                    .join(self.strip_prefix("./").unwrap_or(self));
        } else {
            real_path = self.to_path_buf()
        }
        Some(real_path.to_str()?.into())
    }
}
pub fn path_normalize(path_str: &str) -> String {
    Path::new(path_str).normalize().unwrap()
}

pub fn pick_file() -> String {
    run_fun!(
        ls | fzf;
    ).unwrap_or_else(|_|
        std::process::exit(1)
    )
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
pub fn weekday_from_date(date_str: &str) -> String {
    if date_str.is_empty() { return "".into(); }
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap().weekday().to_string()
}

pub fn match_routine(kind: &str, s_date_str: &str, match_to: &str) -> bool {
    let mut s_date = NaiveDate::parse_from_str(s_date_str, "%Y-%m-%d").unwrap();
    let match_to_date = match match_to {
        "today" => Local::now().date_naive(),
        "yesterday" => Local::now().add(chrono::Duration::days(-1)).date_naive(),
        "tomorrow" => Local::now().add(chrono::Duration::days(1)).date_naive(),
        _ => panic!("unsupported match_to date(only today/yesterday/tomorrow)"),
    };

    if kind == "m" {
        while s_date < match_to_date {
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
        while s_date < match_to_date {
            s_date += chrono::Duration::days(steps);
        }
    }

    s_date == match_to_date
}

pub fn get_box_alias(name_in: &str) -> String {
    match name_in {
        _ if name_in == get_today() => "today",
        _ if name_in == get_tomorrow() => "tomorrow",
        _ if name_in == get_yesterday() => "yesterday",
        _ => name_in,
    }.into()
}

pub fn get_box_unalias(alias: &str) -> String {
    match alias {
        "today" => get_today(),
        "yesterday" => get_yesterday(),
        "tomorrow" => get_tomorrow(),
        "inbox" => taskbox::INBOX_BOXNAME.into(),
        "routine" | "routines" => taskbox::ROUTINE_BOXNAME.into(),
        _ => alias.into(),
    }
}

pub fn get_inbox_file(inbox: &str) -> PathBuf {
    let basedir = PathBuf::from(Config_get!("basedir"));
    let enc_box = basedir.join(inbox).with_extension("mdx");

    if enc_box.exists() { enc_box }
    else { basedir.join(get_box_unalias(inbox)).with_extension("md") }
}

// following i_* fn are for "inquire" based wrappers
// "i" stands for "I would like use Inquire crate to get my Input in an Interactive way"

pub fn i_confirm(question: &str) -> bool {
    inquire::Confirm::new(question)
        .with_default(false)
        .with_render_config(get_confirm_style())
        .prompt().unwrap_or(false)
}

pub fn i_getpass(confirm: bool) -> String {
    let mut com = inquire::Password::new("the password:")
        .with_help_message("<enter> | ctrl+r | ctrl+c")
        .with_render_config(get_pass_input_style())
        .with_display_toggle_enabled();

    if ! confirm { com = com.without_confirmation() }

    execute!(std::io::stdout(), SteadyBar).expect("failed to set cursor");
    let pass = com.prompt().unwrap_or_else(|_| String::new());
    execute!(std::io::stdout(), DefaultUserShape).expect("failed to set cursor");

    pass
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
        assert_eq!(get_box_alias(&get_today()), "today");
        assert_eq!(get_box_alias(&get_yesterday()), "yesterday");
        assert_eq!(get_box_alias(&get_tomorrow()), "tomorrow");
        assert_eq!(get_box_alias("dummy"), "dummy");
        assert_eq!(get_box_alias(""), "");
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
        let op1 = Path::new("~/dummy");
        let op2 = Path::new("dummy");
        let op3 = Path::new("~dummy");
        let op4 = Path::new("./dummy");
        let op5 = Path::new("/dummy");

        assert_eq!(op1.normalize().unwrap(),
            dirs::home_dir().unwrap().join("dummy").to_str().unwrap());
        assert_eq!(op2.normalize().unwrap(),
            std::env::current_dir().unwrap().join("dummy").to_str().unwrap());
        assert_eq!(op3.normalize().unwrap(),
            std::env::current_dir().unwrap().join("~dummy").to_str().unwrap());
        assert_eq!(op4.normalize().unwrap(),
            std::env::current_dir().unwrap().join("dummy").to_str().unwrap());
        assert_eq!(op5.normalize().unwrap(),
            "/dummy");
    }
}
