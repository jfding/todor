use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use colored::Colorize;
use dirs;

use chrono::*;
use std::ops::*;

use crate::util;

const DATA_BASE : &str = ".local/share/todor";
const INBOX_NAME :&str  = "INBOX";

pub fn get_inbox_file(dir: Option<String>, inbox: Option<String>) -> PathBuf {
    let base_path = dir.map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("cannot get home directory")
            .join(DATA_BASE)
    });
    fs::create_dir_all(&base_path).expect("Failed to create base directory");

    base_path.join(inbox.unwrap_or(INBOX_NAME.to_string())).with_extension("md")
}

pub fn get_today() -> String {
    Local::now().date_naive().to_string()
}
pub fn get_yesterday() -> String {
    Local::now().add(chrono::Duration::days(-1)).date_naive().to_string()
}
pub fn get_tomorrow() -> String {
    Local::now().add(chrono::Duration::days(1)).date_naive().to_string()
}

fn get_alias(name_in: Option<String>) -> Option<String> {
    let alias = match name_in {
        Some(name) if name == get_today() => "today",
        Some(name) if name == get_tomorrow() => "tomorrow",
        Some(name) if name == get_yesterday() => "yesterday",
        _ => "",
    };

    if alias.is_empty() { None }
    else { Some(alias.to_string()) }
}

const PREFIX :&str  = "- [ ] ";
const PREFIX_DONE :&str  = "- [x] ";
const SUB_PREFIX :&str  = " 󱞩 ";

#[derive(Debug)]
pub struct TaskBox {
    fpath: PathBuf,
    title: Option<String>,
    alias: Option<String>,
    tasks: Vec<(String, bool)>,
}

impl TaskBox {
    pub fn new (fpath: PathBuf) -> Self {
        let title = fpath.file_stem().and_then(|s| s.to_str()).unwrap().to_string();

        if !fpath.exists() {
            fs::File::create(&fpath).expect("Failed to create file");
            fs::write(&fpath, format!("# {}\n\n", title)).expect("Failed to write to file");
        }

        Self {
            fpath,
            title: None, // None means not loaded
            alias: get_alias(Some(title)),
            tasks: Vec::new(),
        }
    }

    fn _load(&mut self) {
        if self.title.is_some() { return } // avoid _load() twice

        let mut tasks = Vec::new();
        let mut title = String::new();

        let mut postfix_sub = String::new();
        let mut last_is_sub = false;

        for (index, rline) in fs::read_to_string(&self.fpath)
                            .expect("Failed to read file")
                            .lines().enumerate() {

            let line = rline.trim_end();
            if index == 0 {
                title = line.trim_start_matches("# ").to_string();

            } else if line.starts_with("- [") {
                if let Some(stripped) = line.strip_prefix(PREFIX) {
                    tasks.push((stripped.to_string(), false))
                } else if let Some(stripped) = line.strip_prefix(PREFIX_DONE) {
                    tasks.push((stripped.to_string(), true))
                } else { continue }

                if last_is_sub {
                    last_is_sub = false;
                    postfix_sub += " "; // hack way to identify sub-tasks belong to diff task
                }
            } else {
                // might be sub-tasks
                let line = line.trim_start();

                if let Some(stripped) = line.strip_prefix(PREFIX) {
                    tasks.push((SUB_PREFIX.to_owned() + stripped + &postfix_sub, false))
                } else if let Some(stripped) = line.strip_prefix(PREFIX_DONE) {
                    tasks.push((SUB_PREFIX.to_owned() + stripped + &postfix_sub, true))
                } else { continue }

                last_is_sub = true;
            }
        }

        self.title = Some(title);
        self.tasks = tasks;
    }

    fn _dump(&mut self) {
        let mut content = format!("# {}\n\n", self.title.clone().unwrap());

        for (mut task, done) in self.tasks.clone() {
            if let Some(left) = task.strip_prefix(SUB_PREFIX) {
                content.push_str("  ");
                task = left.trim_end().to_string();
            }

            if done {
                content.push_str(PREFIX_DONE)
            } else {
                content.push_str(PREFIX)
            }
            content.push_str(&(task + "\n"))
        }

        fs::write(&self.fpath, content).expect("cannot write file")
    }

    /// clear all uncompelted tasks
    fn _clear(&mut self) {
        self._load();

        let mut newtasks = Vec::new();
        for (task, done) in self.tasks.iter() {
            if *done {
                newtasks.push((task.clone(), *done));
            }
        }

        self.tasks = newtasks;
        self._dump();
    }

    fn _move_in(&mut self, todo_in: &mut TaskBox) {
        self._load();
        todo_in._load();

        let (tasks, _) = todo_in._get_all();
        if tasks.is_empty() { return }

        let from = if todo_in.alias.is_some() {
            todo_in.alias.as_ref()
        } else {
            todo_in.title.as_ref()
        };
        let to = if self.alias.is_some() {
            self.alias.as_ref()
        } else {
            self.title.as_ref()
        };

        println!("{} 󰳟 {} 󰓌", from.unwrap().green(), to.unwrap().blue());

        for task in tasks {
            println!(" {} : {}", " 󰄗".red(), task);
            self.tasks.push((task.clone(), false));
        }

        todo_in._clear();
        self._dump();
    }

    pub fn add(&mut self, what: String, add_date: bool) {
        self._load();

        if add_date { self.tasks.push((format!("{} [󰃵 {}]", what, get_today()), false))
        } else {      self.tasks.push((what, false)) }

        self._dump();
    }

    pub fn _get_all(&mut self) -> (Vec<String> ,Vec<String>) {
        self._load();
        (
            self.tasks.iter().filter(|(_,done)| !done).map(|(task, _)| task.clone()).collect(),
            self.tasks.iter().filter(|(_,done)| *done).map(|(task, _)| task.clone()).collect()
        )
    }
    pub fn list(&mut self, listall: bool) {
        let (tasks, dones) = self._get_all();

        if listall && !dones.is_empty() {
            for t in dones {
                println!("{}  {}", "󰄲".green(), t.strikethrough())
            }
            println!();
        }

        if tasks.is_empty() {
            println!(" {} left!", "nothing".yellow());
        } else {
            let mut msg;
            let mut last_is_sub = false;
            for t in tasks {
                msg = format!("{}  ", "󰄗".blink().blue());
                if t.starts_with(SUB_PREFIX) {
                    msg = format!("{} {}", "󱞩".to_string().blink(), msg);
                    msg += t.strip_prefix(SUB_PREFIX).unwrap();

                    last_is_sub = true;
                } else {
                    if last_is_sub {
                        last_is_sub = false;
                        msg = "\n".to_owned() + &msg;
                    }
                    msg = msg + &t;
                }
                println!("{}", msg);
            }
        }
    }

    pub fn count(&mut self) -> usize {
        self._load();
        self.tasks.iter().filter(|(_, done)| !done).count()
    }

    pub fn mark(&mut self, items: Vec<String>) {
        self._load();

        if items.is_empty() || self.tasks.is_empty() {
            return
        }

        for (task, done) in self.tasks.iter_mut() {
            if *done { continue }
            if items.contains(task) {
                *done = true;
            }
        }

        self._dump();
    }

    pub fn purge(&mut self, sort: bool) {
        use std::collections::HashSet;

        self._load();
        if self.tasks.is_empty() { return }

        // rules: to keep the original order,
        // and when with same content:
        //      done+done => done
        //      not+not => not
        //      done+not => not

        let mut hs = HashSet::new();
        let mut newtasks = Vec::new();

        // 1st scan: remove dups
        for (task, done) in self.tasks.iter() {
            if ! hs.contains(task) {
                newtasks.push((task.clone(), *done));
                hs.insert(task);
            }
        }
        // 2nd scan: check status
        for (task, done) in newtasks.iter_mut() {
            if *done && self.tasks.contains(&(task.to_string(), false)) {
                *done = false
            }
        }

        // (optional) 3rd scan: sort by completed and uncomplated
        // upper: completed
        if sort { newtasks.sort_by(|a, b| b.1.cmp(&a.1)) }

        self.tasks = newtasks;
        self._dump();
    }

    // outdated -> today
    // flag:all -- whether sink future (mainly tomorrow)
    pub fn sink(inbox_path: PathBuf, all: bool) {
        let basedir = inbox_path.as_path().parent().unwrap();
        let mut today_todo = TaskBox::new(basedir.join(get_today()).with_extension("md"));

        let re = Regex::new(r"\d{4}-\d{2}-\d{2}.md$").unwrap();
        let mut boxes = Vec::new();
        for entry in fs::read_dir(basedir).expect("cannot read dir") {
            let path = entry.expect("cannot get entry").path();
            if path.is_file() && re.is_match(path.to_str().unwrap()) { 
                boxes.push(path)
            }
        }
        boxes.sort(); boxes.reverse();

        let today =  Local::now().date_naive();
        for taskbox in boxes {
            let boxdate = NaiveDate::parse_from_str(
                taskbox.file_stem().unwrap().to_str().unwrap(),
                "%Y-%m-%d").expect("something wrong!");

            if boxdate < today || (all && boxdate != today) {
                let mut todo = TaskBox::new(taskbox);
                today_todo._move_in(&mut todo);
            }
        }
    }

    // today -> tomorrow
    pub fn shift(inbox_path: PathBuf) {
        let basedir = inbox_path.as_path().parent().unwrap();

        let mut today_todo = TaskBox::new(basedir.join(get_today()).with_extension("md"));
        let mut tomor_todo = TaskBox::new(basedir.join(get_tomorrow()).with_extension("md"));
        tomor_todo._move_in(&mut today_todo)
    }

    // INBOX -> today
    pub fn collect(inbox_path: PathBuf, inbox_from: Option<String>) {
        let basedir = inbox_path.as_path().parent().unwrap();

        if inbox_from == Some(get_today()) {
            println!("{} is not a valid source", "󰄹 today".red());
            return
        }

        let mut today_todo = TaskBox::new(basedir.join(get_today()).with_extension("md"));
        let mut from_todo = if let Some(from_name) = inbox_from {
            TaskBox::new(basedir.join(from_name).with_extension("md"))
        } else {
            TaskBox::new(basedir.join(INBOX_NAME).with_extension("md"))
        };

        today_todo._move_in(&mut from_todo)
    }

    // today -> INBOX
    pub fn postp(inbox_path: PathBuf) {
        let basedir = inbox_path.as_path().parent().unwrap();

        let mut today_todo = TaskBox::new(basedir.join(get_today()).with_extension("md"));
        let mut inbox_todo = TaskBox::new(basedir.join(INBOX_NAME).with_extension("md"));
        inbox_todo._move_in(&mut today_todo)
    }

    // specified markdown file -> cur
    pub fn import(&mut self, file: Option<String>) {
        let mdfile= file.unwrap_or(util::pick_file());

        let fpath = Path::new(&mdfile);
        if ! fpath.is_file() {
            eprintln!("not a file or not exists: {}", mdfile.red());
            std::process::exit(1)
        }
        println!("importing {} ↩️", mdfile.purple());

        let mut newt = Vec::new();
        for rline in fs::read_to_string(fpath).expect("cannot read file").lines() {
            let line = rline.trim();
            if line.is_empty() { continue }

            if let Some(stripped) = line.strip_prefix(PREFIX) {
                println!(" {} : {}", " 󰄗".red(), stripped);
                newt.push((stripped.to_string(), false))
            }
        }

        if newt.is_empty() {
            println!("{} found", "nothing".yellow())
        } else {
            self._load();
            self.tasks.append(&mut newt);
            self._dump();
        }
    }

    pub fn list_boxes(inbox_path: PathBuf) {
        let basedir = inbox_path.as_path().parent().unwrap();
        println!("[ {} ]", basedir.display().to_string().purple());

        let mut boxes = Vec::new();
        for entry in fs::read_dir(basedir).expect("cannot read dir") {
            let path = entry.expect("cannot get entry").path();
            if path.is_file() && path.extension().unwrap() == "md" {
                boxes.push(String::from(path.file_stem().unwrap().to_str().unwrap()))
            }
        }
        boxes.sort(); boxes.reverse(); boxes.into_iter().for_each(
            |b| {
                print!("{}  {}","󰄹".blue(), b);
                let tbox = TaskBox::new(basedir.join(b).with_extension("md"));
                if tbox.alias.is_some() {
                    println!(" ({})", tbox.alias.unwrap().bright_black().blink())
                } else {
                    println!()
                }
            })
    }
}
