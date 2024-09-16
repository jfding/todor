use std::path::Path;
use std::env;
use cmd_lib::*;
use colored::Colorize;

pub fn glance_all(inbox_path: &Path) {

    let wildpat = format!("{}/*.md", inbox_path.parent().unwrap().display());
    let pager = "fzf --no-sort --tac";

    let res = run_fun!(
      sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager"
    ).unwrap_or_else(|_| String::from("- [ ] n/a"));

    println!("{}", &res[6..])
}

pub fn edit_box(inbox_path: &Path, with: Option<String>) {
    let editor = env::var("EDITOR").unwrap_or("vi".to_string());

    if let Some(other) = with {
        let otherf = format!("{}/{}.md", inbox_path.parent().unwrap().display(), &other);
        println!("editing : {} v.s. {}", inbox_path.display().to_string().purple(), other.red());
        run_cmd!(
            vimdiff $inbox_path $otherf 2>/dev/null
        ).expect("cannot launch vimdiff")

    } else {
        println!("editing : {}", inbox_path.display().to_string().purple());
        run_cmd!(
            $editor $inbox_path 2>/dev/null
        ).expect("cannot launch cli editor(vi?)")
    }
}
