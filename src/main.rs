use clap::{Parser, Subcommand};

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

fn main() {
    let args = Cli::parse();

    println!("{:?}", args);
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
