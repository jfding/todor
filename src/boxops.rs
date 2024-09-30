use std::path::Path;
use cmd_lib::*;
use colored::Colorize;

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
