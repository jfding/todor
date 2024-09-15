use std::path::PathBuf;
use std::fs;
use std::env;
use dirs;
use cmd_lib::*;

const DATA_BASE : &str = ".local/share/todor";

pub fn get_inbox_file(dir: Option<String>, inbox: Option<String>) -> PathBuf {
    let base_path = dir.map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("cannot get home directory")
            .join(DATA_BASE)
    });
    fs::create_dir_all(&base_path).expect("Failed to create base directory");

    return base_path.join(inbox.unwrap_or("TODO".to_string())).with_extension("md");
}

pub fn glance_all(inbox_path: &PathBuf) {

    let wildpat = format!("{}/*.md", inbox_path.as_path().parent().unwrap().display());
    let pager = "fzf --no-sort --tac";

    let res = run_fun!(
      sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager"
    ).unwrap_or_else(|_| String::from("- [ ] n/a"));

    println!("{}", &res[6..])
}

pub fn edit_box(inbox_path: &PathBuf) {
    let editor = env::var("EDITOR").unwrap_or("vi".to_string());
    println!("editing todo box file: {}", inbox_path.display());
    run_cmd!(
        $editor $inbox_path 2>/dev/null
    ).expect("cannot launch cli editor(vi?)")
}
