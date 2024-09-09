use std::path::PathBuf;
use std::fs;
use std::env;
use std::process::Command;
use clap::{Parser, Subcommand};
use dirs;
use inquire::ui::RenderConfig;
use colored::Colorize;

use todor::TaskBox;

#[derive(Debug, Parser)]
#[command(name= "todor")]
#[command(version, about= "yet-another TODO cli in Rust", long_about=None)]
struct Cli {
    /// working dir
    #[arg(short, long, value_name = "FOLDER")]
    dir: Option<String>,

    /// inbox file
    #[arg(short, long, value_name = "FILE")]
    inbox: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// -> add todo item to inbox
    #[clap(visible_alias("a"))]
    Add,
    /// -> list all todo items in inbox
    #[clap(visible_aliases(["l", "ls"]))]
    List,
    /// -> edit todo inbox file
    #[clap(visible_aliases(["e", "ed"]))]
    Edit,
    /// -> count items in inbox
    #[clap(visible_aliases(["c"]))]
    Count,
}

fn get_inbox_file(dir: Option<String>, inbox: Option<String>) -> PathBuf {
    let base_path = dir.map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("cannot get home directory")
            .join(".local/share/todor")
    });
    fs::create_dir_all(&base_path).expect("Failed to create base directory");
    return base_path.join(inbox.unwrap_or("TODO".to_string())).with_extension("md");
}   

fn main() {
    let args = Cli::parse();
    let inbox_path = get_inbox_file(args.dir, args.inbox);

    match args.command {
        Some(Commands::Add) => {
            let todo = TaskBox::new(inbox_path);

            let input = inquire::Text::new("")
                .with_help_message("Enter to add")
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
    }
}