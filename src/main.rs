use std::io;
use colored::Colorize;
use crossterm::execute;
use crossterm::cursor::SetCursorStyle::*;

use todor::taskbox::*;
use todor::cli::*;
use todor::conf::*;
use todor::styles::*;
use todor::util::*;

use todor::util;
use todor::boxops;

fn main() {
    let args = Cli::default();
    let mut inbox = args.inbox;

    let clicmd = std::env::args().next().expect("cannot get arg0");
    if clicmd.ends_with("today") {
        inbox = Some(get_today())
    } else if clicmd.ends_with("tomorrow") {
        inbox = Some(get_tomorrow())
    }

    if args.config.is_some() {
        let mut g_conf = CONFIG.write().unwrap();
        let conf = Config::load(args.config);
        g_conf.update_with(&conf);
    }
    if let Some(dir) = args.dir {
        let mut g_conf = CONFIG.write().unwrap();
        g_conf.basedir = Some(util::path_normalize(&dir));
    }

    let mut inbox_path = util::get_inbox_file(inbox);

    match args.command {
        Some(Commands::List) | None       => TaskBox::new(inbox_path).list(false),
        Some(Commands::Listall)           => TaskBox::new(inbox_path).list(true),
        Some(Commands::Count)             => TaskBox::new(inbox_path).count(),
        Some(Commands::Import{ file })    => TaskBox::new(inbox_path).import(file),
        Some(Commands::Purge { sort }) => {
            if confirm("are you sure?") {
                if sort && ! confirm("sort cannot handle subtasks well, continue?") { return }
                TaskBox::new(inbox_path).purge(sort)
            }
        }

        Some(Commands::Sink { all })      => TaskBox::sink(all),
        Some(Commands::Shift)             => TaskBox::shift(),
        Some(Commands::Collect { inbox }) => TaskBox::collect(inbox),
        Some(Commands::Postp)             => TaskBox::postp(),

        Some(Commands::Mark) => {
            let mut todo = TaskBox::new(inbox_path);
            let tasks = todo._get_all_to_mark();
            if tasks.is_empty() {
                println!(" {} left!", S_empty!("nothing"));
                return
            }

            execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");
            let selected = inquire::MultiSelect::new("choose to close:", tasks)
                .with_render_config(get_multi_select_style())
                .with_vim_mode(true)
                .with_page_size(10)
                .with_help_message("j/k | <space> | <enter> | ctrl+c")
                .prompt().unwrap_or_else(|_| Vec::new());
            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");

            todo.mark(selected);
        }

        Some(Commands::Add { what, date_stamp, routine }) => {
            if routine.is_some() {
                inbox_path = util::get_routine_box_file()
            }

            let mut todo = TaskBox::new(inbox_path);

            execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");
            let mut input = what.unwrap_or_else(|| {
                inquire::Text::new("")
                    .with_render_config(get_text_input_style())
                    .with_help_message("<enter> | ctrl+c")
                    .with_placeholder("something to do?")
                    .prompt().unwrap_or_else(|_| String::new())
            });
            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");

            input = input.trim().to_string();
            if !input.is_empty() {
                todo.add(input, routine, date_stamp);
                println!("{}", S_success!("Task added successfully!"));
            } else {
                println!("{}", S_empty!("Empty input, skip."));
            }
        }

        Some(Commands::Edit { diffwith }) => {
            // just to touch the file
            let _todo = TaskBox::new(inbox_path.clone());

            boxops::edit_box(&inbox_path, diffwith);
        }
        Some(Commands::Glance)  => boxops::glance_all(),
        Some(Commands::Listbox) => boxops::list_boxes(),
        Some(Commands::Cleanup) => boxops::cleanup().expect("failed"),
    }
}
