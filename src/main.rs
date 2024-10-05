use std::path;
use colored::Colorize;

use todor::taskbox::*;
use todor::cli::*;
use todor::conf::*;
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
            if i_confirm("are you sure?") {
                if sort && ! i_confirm("sort cannot handle subtasks well, continue?") { return }
                TaskBox::new(inbox_path).purge(sort)
            }
        }

        Some(Commands::Sink { all })      => TaskBox::sink(all),
        Some(Commands::Shift)             => TaskBox::shift(),
        Some(Commands::Pool)              => TaskBox::pool(),
        Some(Commands::Checkout)          => TaskBox::new(util::get_inbox_file("today"))
                               .collect(&mut TaskBox::new(util::get_inbox_file("routine"))),

        Some(Commands::Collect { boxname, interactive }) => {

            let from = boxname.unwrap_or("inbox".into());
            if from == get_today() || from == "today" {
                println!("{} is not a valid source", S_moveto!("today"));
                return
            }

            let mut tb_from = TaskBox::new(util::get_inbox_file(&from));

            if interactive {
                tb_from.selected = Some(i_select(tb_from.get_all_to_mark()));
            }

            TaskBox::new(util::get_inbox_file("today")).collect(&mut tb_from)
        }

        Some(Commands::Mark) => {
            let mut todo = TaskBox::new(inbox_path);
            let tasks = todo.get_all_to_mark();
            if tasks.is_empty() {
                println!(" {} left!", S_empty!("nothing"));
                return
            }

            todo.mark(i_select(tasks));
        }

        Some(Commands::Add { what, date_stamp, routine, interactive }) => {
            if routine.is_some() {
                inbox_path = get_inbox_file("routine")
            }
            let mut todo = TaskBox::new(inbox_path);

            let input = what.unwrap_or(i_text());
            if !input.is_empty() {
                let mut start_date = get_today();

                if routine.is_some() && interactive {
                    start_date = i_date(match routine {
                            Some(Routine::Daily)    => "daily",
                            Some(Routine::Weekly)   => "weekly",
                            Some(Routine::Biweekly) => "biweekly",
                            Some(Routine::Qweekly)  => "qweekly",
                            Some(Routine::Monthly)  => "monthly",
                            _ => "",
                            })
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
