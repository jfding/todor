use std::path;
use colored::Colorize;
use chrono::*;
use regex::Regex;

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
        Some(Commands::List) | None => TaskBox::new(inbox_path).list(false),
        Some(Commands::Listall)     => TaskBox::new(inbox_path).list(true),
        Some(Commands::Routines)    => TaskBox::new(get_inbox_file(ROUTINE_BOXNAME)).list(true),

        Some(Commands::Count)             => {
            let cc = TaskBox::new(inbox_path).count();
            if cc > 0 { println!("{}", cc) }
        }

        Some(Commands::Import{ file })    => TaskBox::new(inbox_path).import(file),
        Some(Commands::Purge { sort }) => {
            if i_confirm("are you sure?") {
                if sort && ! i_confirm("sort cannot handle subtasks well, continue?") { return }
                TaskBox::new(inbox_path).purge(sort)
            }
        }

        Some(Commands::Checkout) => { // ROUTINE --(check-out)-> today/tomorrow
            let real_inbox = if inbox != "tomorrow" { "today" } else { inbox };
            TaskBox::new(util::get_inbox_file(real_inbox))
                  .collect_from(&mut TaskBox::new(util::get_inbox_file("routine")))
        }

        Some(Commands::Sink { interactive }) => { // outdated -> today
            let basedir = Config_get!("basedir");
            let mut tb_today = TaskBox::new(util::get_inbox_file("today"));

            let mut boxes = Vec::new();
            let re_date_box = Regex::new(r"\d{4}-\d{2}-\d{2}.md$").unwrap();
            for entry in std::fs::read_dir(basedir).expect("cannot read dir") {
                let path = entry.expect("cannot get entry").path();
                if path.is_file() && re_date_box.is_match(path.to_str().unwrap()) { 
                    boxes.push(path)
                }
            }
            boxes.sort(); boxes.reverse();

            let today =  Local::now().date_naive();
            for taskbox in boxes {
                let boxdate = NaiveDate::parse_from_str(
                    taskbox.file_stem().unwrap().to_str().unwrap(),
                    "%Y-%m-%d").expect("something wrong!");

                if boxdate < today {
                    let mut tb_from = TaskBox::new(taskbox);
                    if tb_from.count() == 0 { continue }

                    if interactive {
                        tb_from.selected = Some(i_select(tb_from.get_all_to_mark(),
                                                &format!("choose from {}", boxdate)));
                    }
                    tb_today.collect_from(&mut tb_from);
                    println!();
                }
            }
        }

        Some(Commands::Shift { interactive }) => { // today -> tomorrow
            let mut tb_today = TaskBox::new(util::get_inbox_file("today"));
            if interactive {
                tb_today.selected = Some(i_select(tb_today.get_all_to_mark(), "choose from TODAY"));
            }
            TaskBox::new(util::get_inbox_file("tomorrow")).collect_from(&mut tb_today)
        }

        Some(Commands::Pool { interactive }) => { // today -> INBOX
            let mut tb_today = TaskBox::new(util::get_inbox_file("today"));
            if interactive {
                tb_today.selected = Some(i_select(tb_today.get_all_to_mark(), "choose from TODAY"));
            }

            TaskBox::new(util::get_inbox_file("inbox")).collect_from(&mut tb_today)
        }

        Some(Commands::Collect { from, interactive }) => { // other(def: INBOX) -> today
            let from = from.unwrap_or("inbox".into());
            if from == get_today() || from == "today" {
                println!("{} is not a valid source", S_moveto!("today"));
                return
            }

            let mut tb_from = TaskBox::new(util::get_inbox_file(&from));

            if interactive {
                tb_from.selected = Some(i_select(tb_from.get_all_to_mark(),
                                                 &format!("choose from {}", from)));
            }

            TaskBox::new(util::get_inbox_file("today")).collect_from(&mut tb_from)
        }

        Some(Commands::Mark { delete } ) => {
            let mut todo = TaskBox::new(inbox_path);
            let tasks = todo.get_all_to_mark();
            if tasks.is_empty() {
                println!(" {} left!", S_empty!("nothing"));
                return
            }

            todo.mark(i_select(tasks, "choose to close:"), delete);
        }

        Some(Commands::Add { what, date_stamp, routine, non_interactive }) => {
            if routine.is_some() {
                inbox_path = get_inbox_file("routine")
            }
            let mut todo = TaskBox::new(inbox_path);

            #[allow(clippy::redundant_closure)]
            let input = what.unwrap_or_else(|| i_gettext());
            if ! input.is_empty() {
                let mut start_date = get_today();

                if routine.is_some() && !non_interactive {
                    start_date = i_getdate(match routine {
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

        Some(Commands::Browse)      => boxops::browse().unwrap(),
        Some(Commands::Listbox)     => boxops::list_boxes(),
        Some(Commands::Cleanup)     => boxops::cleanup().unwrap(),
        Some(Commands::Filemanager) => boxops::file_manager().unwrap(),
        Some(Commands::Edit { diffwith, routines }) =>
            boxops::edit_box(if routines { ROUTINE_BOXNAME } else { inbox }, diffwith),
    }
}
