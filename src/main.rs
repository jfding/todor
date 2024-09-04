use clap::{Parser, Subcommand};
use dirs;
use std::path::{Path, PathBuf};
use anyhow::Result;

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

fn get_inbox_file(dir: Option<String>, inbox: Option<String>) -> Result<PathBuf> {
    let mut path = dirs::data_local_dir().expect("cannot get you local data dir");
    if let Some(dir) = dir {
        path = Path::new(&dir).to_path_buf();
    }

    path = path.join(inbox.unwrap_or("TODO".to_string()));
    path.set_extension("md");

    Ok(path)
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args);
    let inbox_path = get_inbox_file(args.dir, args.inbox);
    println!("{:?}", inbox_path);

    match args.command {
        Some(Commands::Add) => {
            println!("todo add item")
        }
        Some(Commands::List) | None => {
            println!("todo list item")
        }
        Some(Commands::Edit) => {
            println!("todo edit item")
        }
        Some(Commands::Count) => {
            println!("todo count item")
        }
    }
}
