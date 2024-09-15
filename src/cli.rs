use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
#[command(name= "todor")]
#[command(version, about= "yet another cli TODO in Rust", long_about=None)]
pub struct Cli {
    /// working dir
    #[arg(short, long, value_name = "FOLDER")]
    pub dir: Option<String>,

    /// inbox file
    #[arg(short, long, value_name = "FILE")]
    pub inbox: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// -> add todo item to inbox
    #[clap(visible_alias("a"))]
    Add,

    /// -> mark item as done
    #[clap(visible_alias("m"))]
    Mark,

    /// -> list all todo items in inbox
    #[clap(visible_aliases(["l", "ls"]))]
    List {
        #[arg(short, long)]
        all: bool,
    },

    /// -> edit todo inbox file
    #[clap(visible_aliases(["e", "ed"]))]
    Edit,

    /// -> count items in inbox
    #[clap(visible_aliases(["c"]))]
    Count,

    /// -> show items in all inboxes
    #[clap(visible_aliases(["A"]))]
    All,

    /// -> purge all the duplicated lines
    Purge, // no alias for safe

    /// -> sink all uncompeleted to "today"
    Sink,
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }
}
