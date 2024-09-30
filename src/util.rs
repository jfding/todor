use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use chrono::*;
use std::ops::*;
use cmd_lib::*;
use colored::Colorize;
use inquire::ui::{ Styled, RenderConfig, Color, StyleSheet, Attributes };
use clap::builder::styling;

pub use crate::*;
pub use crate::conf::*;

pub const CHECKBOX: &str = "󰄗";
pub const CHECKED: &str = "󰄲";
pub const TASKBOX: &str = "󰄹";
pub const MOVING: &str = "󰳟"; // "󰳟" / "" :choose one to be compatible with all popular terms
pub const SCROLLUP: &str = "↥";
pub const SCROLLDOWN: &str = "↧";
pub const SUBTASK: &str = "󱞩";
pub const PROGRESS: &str = "󰓌";
pub const WARN: &str = "󰼈";
pub const QUESTION: &str = "󱜹";

// S means Style
#[macro_export]
macro_rules! S_fpath { ($e:expr) => { $e.to_string().purple() }; }
#[macro_export]
macro_rules! S_checkbox { ($e:expr) => { $e.to_string().blue() }; }
#[macro_export]
macro_rules! S_checked { ($e:expr) => { $e.to_string().green() }; }
#[macro_export]
macro_rules! S_empty { ($e:expr) => { $e.to_string().yellow() }; }
#[macro_export]
macro_rules! S_movefrom { ($e:expr) => { $e.to_string().green() }; }
#[macro_export]
macro_rules! S_moveto { ($e:expr) => { $e.to_string().red() }; }
#[macro_export]
macro_rules! S_hints { ($e:expr) => { $e.to_string().bright_black().blink() }; }
#[macro_export]
macro_rules! S_success { ($e:expr) => { $e.to_string().green().bold() }; }
#[macro_export]
macro_rules! S_failure { ($e:expr) => { $e.to_string().red().blink() }; }

#[macro_export]
macro_rules! S_blink { ($e:expr) => {
    if std::env::var("NO_BLINK").is_ok() || 
       CONFIG.read().unwrap().blink.unwrap_or(true) == false {
        $e.to_string().bold()
    } else {
        $e.to_string().blink()
    }
};}

// for 'clap'
pub fn get_usage_styles() -> styling::Styles {
    styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default())
}

// for 'inquire'
pub fn get_confirm_style() -> RenderConfig<'static> {
    RenderConfig::default()
        .with_prompt_prefix(QUESTION.into())
        .with_answer(
             StyleSheet::default()
            .with_fg(Color::DarkBlue)
            .with_attr(Attributes::BOLD)
        )
}
pub fn get_text_input_style() -> RenderConfig<'static> {
    RenderConfig::default()
        .with_prompt_prefix(CHECKBOX.into())
        .with_answered_prompt_prefix(CHECKBOX.into())
        .with_help_message(
             StyleSheet::default()
            .with_fg(Color::DarkGrey)
            .with_attr(Attributes::ITALIC)
        )
        .with_answer(
             StyleSheet::default()
            .with_fg(Color::DarkBlue)
            .with_attr(Attributes::BOLD)
        )
}
pub fn get_multi_select_style() -> RenderConfig<'static> {
    RenderConfig::default()
        .with_unselected_checkbox(CHECKBOX.into())
        .with_selected_checkbox(CHECKED.into())
        .with_answered_prompt_prefix(CHECKED.into())
        .with_highlighted_option_prefix(MOVING.into())
        .with_scroll_up_prefix(SCROLLUP.into())
        .with_scroll_down_prefix(SCROLLDOWN.into())
        .with_prompt_prefix(
             Styled::new(TASKBOX)
            .with_fg(Color::DarkRed)
         )
        .with_help_message(
             StyleSheet::default()
            .with_fg(Color::DarkGrey)
            .with_attr(Attributes::ITALIC | Attributes::BOLD)
        )
        .with_selected_option(Some(
             StyleSheet::default()
            .with_bg(Color::DarkGrey)
            .with_fg(Color::DarkBlue)
            .with_attr(Attributes::BOLD))
        )
        .with_answer(
             StyleSheet::default()
            .with_fg(Color::DarkBlue)
            .with_attr(Attributes::BOLD)
        )
}

pub fn glance_all(inbox_path: &Path) {
    if cfg!(windows) {
        println!("Sorry, this feature is not supported on Windows.");
        return;
    }

    let wildpat = format!("{}/*.md", inbox_path.parent().unwrap().display());
    let pager = "bat --paging=always -l md";
    let pager_fallback = "less";

    run_cmd!(
      sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager 2>/dev/null"
    ).unwrap_or_else(|_|
        run_cmd!(
          sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager_fallback"
        ).unwrap_or_else(|_| std::process::exit(1)));
}

pub fn edit_box(inbox_path: &Path,
                mut diffwith: Option<String>,
                diffwith_inbox: bool) {
    if diffwith_inbox {
        diffwith = Some(INBOX_NAME.to_string());
    }

    if let Some(other) = diffwith {
        let otherf = if other.ends_with(".md") {
            &other
        } else {
            &format!("{}/{}.md", inbox_path.parent().unwrap().display(), &other)
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

pub fn path_normalize(path_str: String) -> String {
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

pub fn get_box_alias(name_in: Option<String>) -> Option<String> {
    let alias = match name_in {
        Some(name) if name == get_today() => "today",
        Some(name) if name == get_tomorrow() => "tomorrow",
        Some(name) if name == get_yesterday() => "yesterday",
        _ => "",
    };

    if alias.is_empty() { None }
    else { Some(alias.to_string()) }
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
    let basedir = PathBuf::from(CONFIG.read().unwrap().basedir.clone().unwrap());
    fs::create_dir_all(&basedir).expect("Failed to create base directory");

    let inbox_name = match inbox.as_deref() {
        Some("today") => get_today(),
        Some("tomorrow") => get_tomorrow(),
        Some("yesterday") => get_yesterday(),
        None => INBOX_NAME.to_string(),
        _ => inbox.unwrap(),
    };
    basedir.join(inbox_name).with_extension("md")
}
