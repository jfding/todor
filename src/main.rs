use std::io;
use inquire::ui::RenderConfig;
use colored::Colorize;
use crossterm::execute;
use crossterm::cursor::SetCursorStyle::*;

use todor::taskbox::*;
use todor::cli::*;
use todor::util;

fn main() {
    let args = Cli::new();
    let mut inbox = args.inbox;

    let clicmd = std::env::args().nth(0).expect("cannot get arg0");
    if clicmd.ends_with("today") {
        inbox = Some(get_today())
    } else if clicmd.ends_with("tomorrow") {
        inbox = Some(get_tomorrow())
    }

    let inbox_path = util::get_inbox_file(args.dir, inbox);

    match args.command {
        Some(Commands::Mark) | None => {
            let mut todo = TaskBox::new(inbox_path);
            let (tasks,_) = todo._list();
            if tasks.is_empty() {
                println!(" {} left!", "nothing".purple());
                return
            }

            execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");

            let mystyle: RenderConfig = RenderConfig::default()
                .with_unselected_checkbox("󰄗".into())
                .with_selected_checkbox("󰄸".into())
                .with_highlighted_option_prefix("➡️".into())
                .with_scroll_up_prefix("↥".into())
                .with_scroll_down_prefix("↧".into());

            todo.mark(
                inquire::MultiSelect::new("To close:", tasks)
                .with_render_config(mystyle)
                .with_vim_mode(true)
                .with_page_size(10)
                .with_help_message("j/k | <space> | <enter> | ctrl+c")
                .prompt().unwrap_or_else(|_| Vec::new())
            );
            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");
        }

        Some(Commands::List { all }) => {
            let mut todo = TaskBox::new(inbox_path);
            let (tasks, dones) = todo._list();

            if tasks.is_empty() {
                println!(" {} left!", "nothing".purple());
            } else {
                for t in tasks {
                    println!(" 󰄗  {}", t.bold())
                }
            }

            if all {
                println!("");
                for t in dones {
                    println!(" 󰄸  {}", t.strikethrough())
                }
            }
        }

        Some(Commands::Add) => {
            let mut todo = TaskBox::new(inbox_path);

            execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");

            let input = inquire::Text::new("")
                .with_help_message("<enter> | ctrl+c")
                .with_render_config(RenderConfig::default().with_prompt_prefix("✅".into()))
                .with_placeholder("something to do?")
                .prompt().unwrap_or_else(|_| return String::new());

            if !input.is_empty() {
                todo.add(input);
                println!("{}", "Task added successfully!".bold().green());
            } else {
                println!("{}", "No task added. Input was empty.".red());
            }

            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");
        }

        Some(Commands::Edit) => {
            let _todo = TaskBox::new(inbox_path.clone()); // then do nothing, to create the file if it doesn't exist

            util::edit_box(&inbox_path);
        }

        Some(Commands::Count) => {
            let mut todo = TaskBox::new(inbox_path);
            let count = todo.count();
            if count > 0 {
                println!("{}", count);
            }
        }

        Some(Commands::Glance) => {
            util::glance_all(&inbox_path)
        }

        Some(Commands::Purge) => {
            if true == inquire::Confirm::new("are you sure?")
                .with_default(false)
                .prompt().unwrap_or(false) {

                let mut todo = TaskBox::new(inbox_path);
                todo.purge();
            }
        }

        Some(Commands::Sink) => {
            TaskBox::sink(inbox_path.as_path().parent().unwrap())
        }

        Some(Commands::Shift) => {
            TaskBox::shift(inbox_path.as_path().parent().unwrap())
        }

        Some(Commands::Collect) => {
            TaskBox::collect(inbox_path.clone().as_path().parent().unwrap(), inbox_path)
        }

        Some(Commands::Postp) => {
            TaskBox::postp(inbox_path.clone().as_path().parent().unwrap(), inbox_path)
        }
    }
}
