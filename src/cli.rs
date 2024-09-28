use clap::{Parser, Subcommand};
use crate::util;

#[derive(Debug, Clone, Parser)]
#[command(name= "todor")]
#[command(version, about= "yet another cli TODO in Rust", long_about=None)]
#[command(styles=util::get_usage_styles())]
pub struct Cli {
    /// config file
    #[arg(short, long, value_name = "CONF")]
    pub config: Option<String>,

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
    Add {
        #[arg(value_name = "TASK")]
        what: Option<String>,
        #[arg(short, long)]
        date: bool,
    },

    /// -> mark item as done
    #[clap(visible_alias("m"))]
    Mark,

    /// -> list all uncompeleted tasks in box
    #[clap(visible_aliases(["l", "ls"]))]
    List,

    /// -> list all(including compeleted) tasks
    #[clap(visible_aliases(["la"]))]
    Listall,

    /// -> list all todo box in working dir
    #[clap(visible_aliases(["lb"]))]
    Listbox,

    /// -> edit todo inbox file
    #[clap(visible_aliases(["e", "ed"]))]
    Edit {
        #[arg(short, long)]
        #[arg(value_name = "another-taskbox")]
        diffwith: Option<String>,
    },

    /// -> count items in inbox
    #[clap(visible_aliases(["c"]))]
    Count,

    /// -> show items in all inboxes
    #[clap(visible_aliases(["g"]))]
    Glance,

    /// -> purge all the duplicated lines
    Purge {
        #[arg(short, long)]
        sort: bool,
    }, // no alias for safe

    /// -> sink all outdated uncompeleted to "today"
    Sink {
        #[arg(short, long)]
        all: bool,
    },

    /// -> shift all uncompeleted in "today" to "tomorrow"
    Shift,

    /// -> collect all uncompeleted in INBOX(or --inbox <which>) to "today"
    Collect {
        #[arg(short, long)]
        #[arg(value_name = "taskbox-name")]
        inbox: Option<String>,
    },

    /// -> postpone all uncompeleted of today to INBOX
    Postp,

    /// -> import uncompeleted task in any markdown file to current
    Import{
        #[arg(value_name = "markdown-file")]
        file: Option<String>,
    },
}

impl Default for Cli {
    fn default() -> Self {
        Cli::parse()
    }
}
