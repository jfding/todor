use std::path::Path;
use cmd_lib::*;
use colored::Colorize;
use anyhow::Result;
use regex::Regex;
use std::ffi::OsStr;

pub use crate::util::*;

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
        let editor = std::env::var("EDITOR").unwrap_or_else(|_|
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

pub fn list_boxes() {
    let basedir = Config_get!("basedir");
    println!("[ {} ]", S_fpath!(basedir));

    let mut boxes = Vec::new();
    for entry in std::fs::read_dir(&basedir).expect("cannot read dir") {
        let path = entry.expect("cannot get entry").path();
        if path.is_file() && path.extension() == Some(OsStr::new("md")) {
            boxes.push(String::from(path.file_stem().unwrap().to_str().unwrap()))
        }
    }
    boxes.sort(); boxes.reverse(); boxes.into_iter().for_each(
        |b| {
            print!("{}  {}",S_checkbox!(TASKBOX), b);
            if let Some(alias) = get_box_alias(&b) {
                println!(" ({})", S_hints!(alias))
            } else {
                println!()
            }
        })
}

// clean up all empty datetime taskbox
pub fn cleanup() -> Result<()> {
    let basedir = Config_get!("basedir");
    println!("[ {} ]", S_fpath!(basedir));

    let mut boxes = Vec::new();
    let re = Regex::new(r"\d{4}-\d{2}-\d{2}.md$").unwrap();
    for entry in std::fs::read_dir(basedir)? {
        let path = entry.expect("cannot get dir entry").path();
        if path.is_file() && re.is_match(path.to_str().unwrap()) {
            let content = std::fs::read_to_string(&path)?;
            if content.lines().count() <= 2 {
                boxes.push((String::from(path.file_stem().unwrap().to_str().unwrap()), path))
            }
        }
    }
    if boxes.is_empty() {
        println!("{} to cleanup", S_empty!("nothing"));
        return Ok(())
    }

    boxes.sort_by(|a,b| b.0.cmp(&a.0));
    boxes.clone().into_iter().for_each(
        |(name, _path)| {
            print!("{}  {}",S_checkbox!(TASKBOX), name);
            if let Some(alias) = get_box_alias(&name) {
                println!(" ({})", S_hints!(alias))
            } else {
                println!()
            }
        });
    if inquire::Confirm::new("Going to remove the aboves, are you sure?")
        .with_default(false)
        .with_render_config(get_confirm_style())
        .prompt().unwrap_or(false) {
        boxes.into_iter().for_each(
            |(_, path)| {
                std::fs::remove_file(&path).expect("cannot remove file");
                println!("{} removed!", S_fpath!(path.display()));
            }
        );
    }

    Ok(())
}
