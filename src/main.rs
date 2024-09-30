use std::io;
use colored::Colorize;
use crossterm::execute;
use crossterm::cursor::SetCursorStyle::*;

use todor::taskbox::*;
use todor::cli::*;
use todor::conf::*;
use todor::util;
use todor::util::*;

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
        g_conf.basedir = Some(util::path_normalize(dir));
    }

    let inbox_path = util::get_inbox_file(inbox);

    match args.command {
        Some(Commands::List) | None => {
            let mut todo = TaskBox::new(inbox_path);
            todo.list(false);
        }
        Some(Commands::Listall) => {
            let mut todo = TaskBox::new(inbox_path);
            todo.list(true);
        }

        Some(Commands::Mark) => {
            let mut todo = TaskBox::new(inbox_path);
            let tasks = todo._get_all_to_mark();
            if tasks.is_empty() {
                println!(" {} left!", S_empty!("nothing"));
                return
            }

            execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");
            let selected = inquire::MultiSelect::new("choose to close:", tasks)
                .with_render_config(util::get_multi_select_style())
                .with_vim_mode(true)
                .with_page_size(10)
                .with_help_message("j/k | <space> | <enter> | ctrl+c")
                .prompt().unwrap_or_else(|_| Vec::new());
            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");

            todo.mark(selected);
        }

        Some(Commands::Add { what, date }) => {
            let mut todo = TaskBox::new(inbox_path);

            if let Some(input) = what {
                todo.add(input, date);
                println!("{}", S_success!("Task added successfully!"));
                return
            }

            execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");
            let input = inquire::Text::new("")
                .with_render_config(util::get_text_input_style())
                .with_help_message("<enter> | ctrl+c")
                .with_placeholder("something to do?")
                .prompt().unwrap_or_else(|_| String::new());
            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");

            if !input.is_empty() {
                todo.add(input, date);
                println!("{}", S_success!("Task added successfully!"));
            } else {
                println!("{}", S_empty!("No task added. Input was empty."));
            }
        }

        Some(Commands::Edit { diffwith }) => {
            let _todo = TaskBox::new(inbox_path.clone()); // then do nothing, to create the file if it doesn't exist

            todor::util::edit_box(&inbox_path, diffwith);
        }

        Some(Commands::Count) => {
            let mut todo = TaskBox::new(inbox_path);
            let count = todo.count();
            if count > 0 {
                println!("{}", count);
            }
        }

        Some(Commands::Glance) => {
            todor::util::glance_all()
        }

        Some(Commands::Listbox) => {
            TaskBox::list_boxes()
        }

        Some(Commands::Purge { sort }) => {
            if inquire::Confirm::new("are you sure?")
                .with_default(false)
                .with_render_config(util::get_confirm_style())
                .prompt().unwrap_or(false) {
                if sort && ! inquire::Confirm::new("Sort can only work well without subtasks, continue?")
                        .with_default(false)
                        .with_render_config(util::get_confirm_style())
                        .prompt().unwrap_or(false) { return }

                let mut todo = TaskBox::new(inbox_path);
                todo.purge(sort);
            }
        }

        Some(Commands::Sink { all }) => {
            TaskBox::sink(all)
        }

        Some(Commands::Shift) => {
            TaskBox::shift()
        }

        Some(Commands::Collect { inbox }) => {
            TaskBox::collect(inbox)
        }

        Some(Commands::Postp) => {
            TaskBox::postp()
        }

        Some(Commands::Import{ file }) => {
            let mut todo = TaskBox::new(inbox_path);
            todo.import(file)
        }

        Some(Commands::Cleanup) => {
            TaskBox::cleanup().expect("failed")
        }
    }
}
