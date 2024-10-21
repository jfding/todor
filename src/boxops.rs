use cmd_lib::*;
use colored::Colorize;
use anyhow::Result;
use regex::Regex;
use which::which;
use std::ffi::OsStr;
use std::path::{PathBuf, Path};
use chrono::*;

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
      sh -c "$fm $basedir 2>/dev/null"
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
        if path.is_file() {
            if path.extension() == Some(OsStr::new("md")) {
                boxes.push((String::from(path.file_stem().unwrap().to_str().unwrap()), false))
            } else if path.extension() == Some(OsStr::new("mdx")) {
                boxes.push((String::from(path.file_stem().unwrap().to_str().unwrap()), true))
            }
        }
    }
    boxes.sort_by(|a,b| b.0.cmp(&a.0));
    boxes.into_iter().for_each(
        |(boxname, encrypted)| {
            if encrypted {
                print!("{} ", S_warning!(LOCKED));
            } else {
                print!("  ");

            }
            print!("{}  {}",S_checkbox!(TASKBOX), boxname);
            if let Some(alias) = get_box_alias(&boxname) {
                println!(" ({})", S_hints!(alias))
            } else {
                println!()
            }
        })
}

pub fn zip_file_with_pass(ifile: &Path, ofile: &Path, passwd: &str) {
    let cmd = format!("cd {}; zip -P {} {} {} >/dev/null; rm -f {}",
        ifile.parent().unwrap().display(),
        passwd,
        ofile.display(),
        ifile.file_name().unwrap().to_str().unwrap(),
        ifile.display()
        );
    run_cmd!(sh -c $cmd).expect("cannot zip file");
}
fn unzip_file_with_pass(ifile: &Path, passwd: &str) {
    let cmd = format!("cd {}; unzip -P {} {} >/dev/null; rm -f {}",
        ifile.parent().unwrap().display(),
        passwd,
        ifile.file_name().unwrap().to_str().unwrap(),
        ifile.display()
        );
    run_cmd!(sh -c $cmd).expect("cannot unzip file");
}

pub fn enc_boxfile(ifile: &Path) {
    let tbname = ifile.file_stem().unwrap().to_str().unwrap();

    // validating ext name
    if ifile.extension() == Some(OsStr::new("mdx")) {
        println!("Taskbox: {} was already encrypted, skipped", S_checkbox!(tbname));
        std::process::exit(1);
    }

    // validating box name: reserved and date format box cannot enc
    let can_be = match tbname {
        ROUTINE_BOXNAME | INBOX_BOXNAME => false,
        _ if Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap().is_match(tbname) => false,
        _ => true
    };
    if ! can_be {
        println!("Taskbox: {} cannot be encrypted, skipped", S_checkbox!(tbname));
        std::process::exit(1);
    }

    let passwd = i_getpass(true);
    if passwd.is_empty() {
        println!("password is empty, canceled");
        std::process::exit(1);
    }

    println!("Encrypting taskbox: {}", S_checkbox!(tbname));

    let mut encfile = PathBuf::from(ifile);
    encfile.set_extension("mdx");
    zip_file_with_pass(ifile, &encfile, &passwd)
}

pub fn dec_boxfile(ifile: &Path) {
    let tbname = ifile.file_stem().unwrap().to_str().unwrap();
    // validating ext name
    if ifile.extension() == Some(OsStr::new("md")) {
        println!("Taskbox: {} was not encrypted, skipped", S_checkbox!(tbname));
        std::process::exit(1);
    }

    let passwd = i_getpass(false);
    if passwd.is_empty() {
        println!("password is empty, canceled");
        std::process::exit(1);
    }

    println!("Decrypting taskbox: {}", S_checkbox!(ifile.file_stem().unwrap().to_str().unwrap()));

    unzip_file_with_pass(ifile, &passwd)
}

// clean up and all empty datetime taskbox and archive done tasks
// rules:
// 1. all empty boxed will be removed
// 2. all boxes with only DONE tasks will be removed and the tasks go ARCHIVE box
// 3. will keep "yesterday" "today" "tomorrow" untouched
pub fn cleanup_and_archive() -> Result<()> {
    let basedir = Config_get!("basedir");

    let mut actions = Vec::new();
    let yesterday = Local::now().date_naive() - Duration::days(1);
    let re = Regex::new(r"(\d{4}-\d{2}-\d{2}).md$").unwrap();
    for entry in std::fs::read_dir(&basedir)? {
        let path = entry.expect("cannot get dir entry").path();
        if path.is_file() {
            if let Some(caps) = re.captures(path.to_str().unwrap()) {
                let boxdate = NaiveDate::parse_from_str(&caps[1], "%Y-%m-%d").unwrap();
                if boxdate < yesterday {
                    let mut tb = TaskBox::new(path.clone());
                    if tb.count() > 0 { continue }
                    if tb.tasks.is_empty() {
                        actions.push(("delete", String::from(path.file_stem().unwrap().to_str().unwrap()), path))
                    } else {
                        actions.push(("archive", String::from(path.file_stem().unwrap().to_str().unwrap()), path))

                    }
                }
            }
        }
    }

    if actions.is_empty() {
        println!("{} to cleanup and archive, skipped", S_empty!("nothing"));
        return Ok(())
    }

    let archive_dir = Path::new(&basedir).join("archives");
    if ! archive_dir.exists() {
        std::fs::create_dir(&archive_dir).expect("cannot create archive dir");
    }

    actions.sort_by(|a,b| b.1.cmp(&a.1)); // reverse ordering by date

    actions.clone().into_iter().for_each(
        |(act, name, _path)| {
            println!("{}  {} ::{}", S_checkbox!(TASKBOX), name, S_blink!(S_warning!(act)));
        }
     );

    if util::i_confirm("Going to apply the above actions, are you sure?") {
        actions.into_iter().for_each(
            |(act, _name, path)| {
                if act == "archive" {
                    std::fs::rename(&path, archive_dir.join(path.file_name().unwrap())).expect("cannot move file");
                } else {
                    std::fs::remove_file(&path).expect("cannot remove file");
                }
            }
        );
    }

    Ok(())
}
