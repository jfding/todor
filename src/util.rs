use std::path::Path;
use std::env;
use cmd_lib::*;
use colored::Colorize;
use inquire::ui::{ Styled, RenderConfig, Color, StyleSheet, Attributes };

pub const CHECKBOX: &str = "󰄗";
pub const CHECKED: &str = "󰄲";
pub const TASKBOX: &str = "󰄹";
pub const MOVING: &str = "󰳟"; // "󰳟" / "" :choose one to be compatible with all popular terms
pub const SCROLLUP: &str = "↥";
pub const SCROLLDOWN: &str = "↧";
pub const SUBTASK: &str = "󱞩";
pub const PROGRESS: &str = "󰓌";

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
macro_rules! S_hints { ($e:expr) => { $e.to_string().bright_black() }; }
#[macro_export]
macro_rules! S_success { ($e:expr) => { $e.to_string().green().bold() }; }
#[macro_export]
macro_rules! S_failure { ($e:expr) => { $e.to_string().red().blink() }; }

pub use S_fpath;
pub use S_checkbox;
pub use S_checked;
pub use S_empty;
pub use S_movefrom;
pub use S_moveto;
pub use S_hints;
pub use S_success;
pub use S_failure;

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
    let prompt_prefix = Styled::new(TASKBOX).with_fg(Color::DarkRed);

    RenderConfig::default()
        .with_unselected_checkbox(CHECKBOX.into())
        .with_selected_checkbox(CHECKED.into())
        .with_answered_prompt_prefix(CHECKED.into())
        .with_highlighted_option_prefix(MOVING.into())
        .with_scroll_up_prefix(SCROLLUP.into())
        .with_scroll_down_prefix(SCROLLDOWN.into())
        .with_prompt_prefix(prompt_prefix)
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

pub fn edit_box(inbox_path: &Path, diffwith: Option<String>) {
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
