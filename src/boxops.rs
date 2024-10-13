use cmd_lib::*;
use colored::Colorize;
use anyhow::Result;
use regex::Regex;
use which::which;
use std::ffi::OsStr;

use crate::util::*;
use crate::taskbox::*;

pub fn browse() -> Result<()> {
    if cfg!(windows) {
        println!("Sorry, this feature is not supported on Windows.");
        return Ok(());
    }

    let wildpat = format!("{}/*.md", Config_get!("basedir"));
    let pager = which("glow").unwrap_or(
                which("bat").unwrap_or(
                which("less").unwrap_or(
                which("more")?)));
    let pager_args = match pager.to_str().unwrap() {
        p if p.ends_with("glow") => "--style=dark | less -r",
        p if p.ends_with("bat") => "-l md --paging=never",
        p if p.ends_with("less") => "-r",
        _ => ""
    };

    run_cmd!(
      sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager $pager_args 2>/dev/null"
    )?;
    Ok(())
}

pub fn file_manager() -> Result<()> {
    if cfg!(windows) {
        println!("Sorry, this feature is not supported on Windows.");
        return Ok(());
    }

    let basedir = Config_get!("basedir");
    let def_fm = if cfg!(target_os = "macos") {
        "open"
    } else {
        // linux
        "/bin/ls -l"
    };

    let fm = which("ranger").unwrap_or(def_fm.into());

    run_cmd!(
      sh -c "$fm $basedir"
    )?;
    Ok(())
}

pub fn edit_box(cur_box: &str, diffwith: Option<String>) {
    let boxpath = get_inbox_file(cur_box);
    _ = TaskBox::new(boxpath.clone()); // only touch file

    if let Some(other) = diffwith {
        let otherf = if other.ends_with(".md") {
            &other
        } else {
            &format!("{}/{}.md", Config_get!("basedir"), get_box_unalias(&other))
        };

        println!("editing : {} v.s. {}", S_fpath!(boxpath.display()), S_fpath!(otherf));
        if run_cmd!(
            vimdiff $boxpath $otherf 2>/dev/null
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

        println!("editing : {}", S_fpath!(boxpath.display()));
        run_cmd!(
            $editor $boxpath 2> $nulldev
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
    if util::i_confirm("Going to remove the aboves, are you sure?") {
        boxes.into_iter().for_each(
            |(_, path)| {
                std::fs::remove_file(&path).expect("cannot remove file");
                println!("{} removed!", S_fpath!(path.display()));
            }
        );
    }

    Ok(())
}
