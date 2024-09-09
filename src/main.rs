use std::path::PathBuf;
use std::fs;
use std::env;
use std::process::Command;
use clap::{Parser, Subcommand};
use dirs;

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
    println!("{:?}", args);
    let inbox_path = get_inbox_file(args.dir, args.inbox);
    println!("inbox file: {:?}", inbox_path);

    match args.command {
        Some(Commands::Add) => {
            let todo = TaskBox::new(inbox_path);

            use std::io::{self, Write};

            print!("Enter a new task: ");
            io::stdout().flush().expect("Failed to flush stdout");

            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            let input = input.trim().to_string();

            if !input.is_empty() {
                todo.add(input);
                println!("Task added successfully!");
            } else {
                println!("No task added. Input was empty.");
            }  
        }

        Some(Commands::List) | None => {
            let todo = TaskBox::new(inbox_path);
            todo.list(None)
        }
        Some(Commands::Edit) => {
            let todo = TaskBox::new(inbox_path.clone()); // then do nothing, to create the file if it doesn't exist

            let editor = env::var("EDITOR").unwrap_or("vi".to_string());
            let mut child = Command::new(editor).arg(&inbox_path).spawn().expect("Failed to start editor");
            child.wait().expect("Failed to wait on editor");
        }
        Some(Commands::Count) => {
            println!("todo count item")
        }
    }
}