use std::env;
use std::ops::Add;
use std::process::Command;
use std::path::PathBuf;
use inquire::ui::RenderConfig;
use colored::Colorize;
use chrono::prelude::*;

use todor::taskbox::TaskBox;
use todor::cli::*;

fn main() {
    let args = Cli::new();
    let mut inbox = args.inbox;

    let clicmd = std::env::args().nth(0).expect("cannot get arg0");
    if clicmd.ends_with("today") {
        inbox = Some(Local::now().format("%Y-%m-%d").to_string());
    } else if clicmd.ends_with("tomorrow") {
        inbox = Some(Local::now().add(chrono::Duration::days(1)).format("%Y-%m-%d").to_string());
    }

    let inbox_path = todor::get_inbox_file(args.dir, inbox);

    match args.command {
        Some(Commands::Add) => {
            let todo = TaskBox::new(inbox_path);

            let input = inquire::Text::new("")
                .with_help_message("<enter> | ctrl+c")
                .with_render_config(RenderConfig::default().with_prompt_prefix("✅".into()))
                .with_placeholder("something to do?")
                .prompt().unwrap_or_else(|_| return String::new());

            if !input.is_empty() {
                todo.add(input);
                println!("{}", "Task added successfully!".bold().green());
            } else {
                println!("{}", "No task added. Input was empty.".red());
            }  
        }

        Some(Commands::List) | None => {
            let mut todo = TaskBox::new(inbox_path);
            let tasks = todo.list(Some(false)); // false means NOT all

            todo.mark(
                inquire::MultiSelect::new("To close:", tasks)
                .with_vim_mode(true)
                .with_help_message("j/k | <space> | <enter> | ctrl+c")
                .prompt().unwrap_or_else(|_| Vec::new())
            )
        }

        Some(Commands::Edit) => {
            let _todo = TaskBox::new(inbox_path.clone()); // then do nothing, to create the file if it doesn't exist

            let editor = env::var("EDITOR").unwrap_or("vi".to_string());
            let mut child = Command::new(editor).arg(&inbox_path).spawn().expect("Failed to start editor");
            child.wait().expect("Failed to wait on editor");
        }

        Some(Commands::Count) => {
            let todo = TaskBox::new(inbox_path);
            let count = todo.count();
            if count > 0 {
                println!("{}", count);
            }
        }

        Some(Commands::All) => {
            show_all(&inbox_path)
        }
    }
}

fn show_all(inbox_path: &PathBuf) {
    use cmd_lib::run_fun;

    let wildpat = format!("{}/*.md", inbox_path.as_path().parent().unwrap().display());
    let pager = "fzf --no-sort --tac";

    let res = run_fun!(
      sh -c "cat $wildpat | sed  's/^#/\\n✅/' | $pager"
    ).unwrap_or_else(|_| String::from("- [ ] n/a"));

    println!("{}", &res[6..])
}
