use std::io;
use inquire::ui::{ Styled, RenderConfig, Color, StyleSheet, Attributes };
use colored::Colorize;
use crossterm::execute;
use crossterm::cursor::SetCursorStyle::*;

use todor::taskbox::*;
use todor::cli::*;
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

    let inbox_path = get_inbox_file(args.dir, inbox);

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
            let (tasks,_) = todo._get_all();
            if tasks.is_empty() {
                println!(" {} left!", S_empty!("nothing"));
                return
            }

            execute!(io::stdout(), BlinkingBlock).expect("failed to set cursor");

            let help_msg_style_sheet =StyleSheet::default()
                .with_fg(Color::DarkGrey)
                .with_attr(Attributes::ITALIC | Attributes::BOLD);

            let selected_opt_style_sheet =StyleSheet::default()
                .with_bg(Color::DarkGrey)
                .with_fg(Color::DarkBlue)
                .with_attr(Attributes::BOLD);

            let answer_style_sheet =StyleSheet::default()
                .with_fg(Color::DarkBlue)
                .with_attr(Attributes::BOLD);

            let prompt_prefix = Styled::new(TASKBOX).with_fg(Color::DarkRed);

            let mystyle: RenderConfig = RenderConfig::default()
                .with_unselected_checkbox(CHECKBOX.into())
                .with_selected_checkbox(CHECKED.into())
                .with_highlighted_option_prefix(MOVING.into())
                .with_scroll_up_prefix(SCROLLUP.into())
                .with_help_message(help_msg_style_sheet)
                .with_selected_option(Some(selected_opt_style_sheet))
                .with_answer(answer_style_sheet)
                .with_prompt_prefix(prompt_prefix)
                .with_scroll_down_prefix(SCROLLDOWN.into());

            todo.mark(
                inquire::MultiSelect::new("choose to close:", tasks)
                .with_render_config(mystyle)
                .with_vim_mode(true)
                .with_page_size(10)
                .with_help_message(" j/k | <space> | <enter> | ctrl+c ")
                .prompt().unwrap_or_else(|_| Vec::new())
            );
            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");
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
                .with_help_message("<enter> | ctrl+c")
                .with_render_config(RenderConfig::default().with_prompt_prefix(CHECKBOX.into()))
                .with_placeholder("something to do?")
                .prompt().unwrap_or_else(|_| String::new());

            if !input.is_empty() {
                todo.add(input, date);
                println!("{}", S_success!("Task added successfully!"));
            } else {
                println!("{}", S_empty!("No task added. Input was empty."));
            }

            execute!(io::stdout(), DefaultUserShape).expect("failed to set cursor");
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
            todor::util::glance_all(&inbox_path)
        }

        Some(Commands::Listbox) => {
            TaskBox::list_boxes(inbox_path)
        }

        Some(Commands::Purge { sort }) => {
            if inquire::Confirm::new("are you sure?")
                .with_default(false)
                .prompt().unwrap_or(false) {

                let mut todo = TaskBox::new(inbox_path);
                todo.purge(sort);
            }
        }

        Some(Commands::Sink { all }) => {
            TaskBox::sink(inbox_path, all)
        }

        Some(Commands::Shift) => {
            TaskBox::shift(inbox_path)
        }

        Some(Commands::Collect { inbox }) => {
            TaskBox::collect(inbox_path, inbox)
        }

        Some(Commands::Postp) => {
            TaskBox::postp(inbox_path)
        }

        Some(Commands::Import{ file }) => {
            let mut todo = TaskBox::new(inbox_path);
            todo.import(file)
        }
    }
}
