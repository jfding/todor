use std::path::Path;
use std::env;
use cmd_lib::*;
use colored::Colorize;

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
macro_rules! S_success { ($e:expr) => { $e.to_string().blue().bold() }; }
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

pub fn glance_all(inbox_path: &Path) {

    let wildpat = format!("{}/*.md", inbox_path.parent().unwrap().display());
    let pager = "fzf --no-sort --tac";

    let res = run_fun!(
      sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager"
    ).unwrap_or_else(|_| String::from("- [ ] n/a"));

    println!("{}", &res[6..])
}

pub fn edit_box(inbox_path: &Path, diffwith: Option<String>) {
    let editor = env::var("EDITOR").unwrap_or("vi".to_string());

    if let Some(other) = diffwith {
        let otherf = if other.ends_with(".md") {
            &other
        } else {
            &format!("{}/{}.md", inbox_path.parent().unwrap().display(), &other)
        };

        println!("editing : {} v.s. {}", S_fpath!(inbox_path.display()), S_fpath!(otherf));
        run_cmd!(
            vimdiff $inbox_path $otherf 2>/dev/null
        ).expect("cannot launch vimdiff")

    } else {
        println!("editing : {}", S_fpath!(inbox_path.display()));
        run_cmd!(
            $editor $inbox_path 2>/dev/null
        ).expect("cannot launch cli editor(vi?)")
    }
}

pub fn pick_file() -> String {
    run_fun!(
        ls | fzf;
    ).ok().unwrap_or_else(||
        std::process::exit(1)
    )
}
