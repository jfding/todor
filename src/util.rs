use std::path::Path;
use std::env;
use cmd_lib::*;
use colored::Colorize;

pub fn glance_all(inbox_path: &Path) {
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
    let editor = env::var("EDITOR").unwrap_or("vi".to_string());

    if let Some(other) = diffwith {
        let otherf = if other.ends_with(".md") {
            &other
        } else {
            &format!("{}/{}.md", inbox_path.parent().unwrap().display(), &other)
        };

        println!("editing : {} v.s. {}", inbox_path.display().to_string().purple(), other.red());
        run_cmd!(
            vimdiff $inbox_path $otherf 2>/dev/null
        ).expect("cannot launch vimdiff")

    } else {
        println!("editing : {}", inbox_path.display().to_string().purple());
        run_cmd!(
            $editor $inbox_path 2>/dev/null
        ).expect("cannot launch $EDITOR or vi")
    }
}

pub fn pick_file() -> String {
    run_fun!(
        ls | fzf;
    ).unwrap_or_else(|_|
        std::process::exit(1)
    )
}
