use inquire::ui::{ Styled, RenderConfig, Color, StyleSheet, Attributes, calendar };
use clap::builder::styling;

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
pub const ROUTINES: &str = "󰃯";
pub const CALENDAR: &str = "󰃵";
pub const WEEKLINE: &str = "󰕶";
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
macro_rules! S_warning { ($e:expr) => { $e.to_string().yellow() }; }
#[macro_export]
macro_rules! S_routine { ($e:expr) => { $e.to_string().purple().italic() }; }
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
pub fn get_date_input_style() -> RenderConfig<'static> {
    let mut cal_conf = calendar::CalendarRenderConfig::default_colored()
            .with_prefix(
                 Styled::new(WEEKLINE)
                .with_fg(Color::DarkGrey)
            );
    cal_conf.week_header = StyleSheet::default()
            .with_bg(Color::DarkGrey)
            .with_attr(Attributes::ITALIC);

    RenderConfig::default()
        .with_prompt_prefix(
             Styled::new(ROUTINES)
            .with_fg(Color::LightYellow)
         )
        .with_answered_prompt_prefix(ROUTINES.into())
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
        .with_calendar_config(cal_conf)
}
