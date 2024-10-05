use std::io;
use std::path;
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
    let arg0 = std::env::args().next().unwrap();

    let inbox =
        if let Some(boxname) = args.inbox {
            &boxname.clone()
        } else {
            let cmdname = arg0.split(path::MAIN_SEPARATOR).last().unwrap();
            if cmdname == "todor" {
                "inbox"
            } else {
                // e.g. "today", "tomorrow", "yesterday", "t.read", "todo.working"
                cmdname.split('.').last().unwrap()
            }
        };

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
        Some(Commands::Pool)              => TaskBox::pool(),
        Some(Commands::Checkout)          => TaskBox::new(util::get_inbox_file("today"))
                                                            .collect("routine", vec![]),
        Some(Commands::Collect { boxname, interactive }) => {

            let from = boxname.unwrap_or("inbox".into());
            if from == get_today() || from == "today" {
                println!("{} is not a valid source", S_moveto!("today"));
                return
            }

            let mut selected = vec![];
            if interactive {
                let tasks = TaskBox::new(util::get_inbox_file(&from)).get_all_to_mark();

                execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");
                selected = inquire::MultiSelect::new("choose to move:", tasks)
                    .with_render_config(get_multi_select_style())
                    .with_vim_mode(true)
                    .with_page_size(10)
                    .with_help_message("j/k | <space> | <enter> | ctrl+c")
                    .prompt().unwrap_or_else(|_| Vec::new());
                execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");
            }

            TaskBox::new(util::get_inbox_file("today")).collect(&from, selected)
        }

        Some(Commands::Mark) => {
            let mut todo = TaskBox::new(inbox_path);
            let tasks = todo.get_all_to_mark();
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

        Some(Commands::Add { what, date_stamp, routine, interactive }) => {
            if routine.is_some() {
                inbox_path = get_inbox_file("routine")
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
                let mut start_date = get_today();
                if routine.is_some() && interactive {
                    let kind= match routine {
                            Some(Routine::Daily)    => "daily",
                            Some(Routine::Weekly)   => "weekly",
                            Some(Routine::Biweekly) => "biweekly",
                            Some(Routine::Qweekly)  => "qweekly",
                            Some(Routine::Monthly)  => "monthly",
                            _ => "",
                            };

                    start_date = inquire::DateSelect::new(&format!(" {} from:",S_routine!(kind)))
                        .with_render_config(get_date_input_style())
                        .with_help_message("h/j/k/l | <enter> | ctrl+c")
                        .prompt().unwrap_or_else(|_| {
                                println!("{}", S_empty!("No starting date selected, skip."));
                                std::process::exit(1)
                            }).to_string();
                }
                todo.add(input, routine, date_stamp, &start_date);
                println!("{}", S_success!("Task added successfully!"));
            } else {
                println!("{}", S_empty!("Empty input, skip."));
            }
        }

        Some(Commands::Browse)  => boxops::browse().unwrap(),
        Some(Commands::Listbox) => boxops::list_boxes(),
        Some(Commands::Cleanup) => boxops::cleanup().unwrap(),
        Some(Commands::Edit { diffwith, routines }) =>
            boxops::edit_box(if routines { ROUTINE_BOXNAME } else { inbox }, diffwith),
    }
}
