use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use colored::Colorize;
use dirs;

use chrono::*;
use std::ops::*;

use crate::util;
use crate::util::*;
use crate::conf::*;

pub fn get_inbox_file(dir: Option<String>, inbox: Option<String>) -> PathBuf {
    // for windows compatibility
    let rel_base :PathBuf = DATA_BASE.split("/").collect();

    let base_path = dir.map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("cannot get home directory")
            .join(rel_base)
    });
    fs::create_dir_all(&base_path).expect("Failed to create base directory");

    let inbox_name = match inbox.as_deref() {
        Some("today") => get_today(),
        Some("tomorrow") => get_tomorrow(),
        Some("yesterday") => get_yesterday(),
        None => INBOX_NAME.to_string(),
        _ => inbox.unwrap(),
    };
    base_path.join(inbox_name).with_extension("md")
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

    /// drain all uncompelted tasks
    fn _drain(&mut self) {
        self._load();

        let mut newtasks = Vec::new();
        let mut last_major_task: Option<(String, bool)> = None;

        for (task, done) in self.tasks.iter() {
            if task.starts_with(SUB_PREFIX) {
                if *done {
                    if let Some((ref last_major, lm_done)) = last_major_task {
                        if !lm_done {
                            newtasks.push((last_major.to_string(), true));
                            last_major_task = None;
                        }
                    }
                }
            } else {
                last_major_task = Some((task.clone(), *done));
            }

            if *done {
                newtasks.push((task.clone(), true));
            }
        }

        self.tasks = newtasks;
        self._dump();
    }

    fn _move_in(&mut self, todo_in: &mut TaskBox) {
        self._load();
        todo_in._load();

        let tasks = todo_in._get_all_to_mark();
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

        println!("{} {} {} {}", S_movefrom!(from.unwrap()), MOVING, S_moveto!(to.unwrap()), PROGRESS);

        for task in tasks {
            if task.contains(WARN) {
                println!("  {} : {}", S_checkbox!(CHECKED), task);
                self.tasks.push((task.clone(), true));
            } else {
                println!("  {} : {}", S_checkbox!(CHECKBOX), task);
                self.tasks.push((task.clone(), false));
            }
        }

        todo_in._drain();
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
    pub fn _get_all_to_mark(&mut self) -> Vec<String> {
        self._load();

        let mut tasks = Vec::new();
        let mut last_major_task :Option<(String, bool)> = None;
        for (t, done) in &self.tasks {
            if t.starts_with(SUB_PREFIX) {
                if let Some((ref last_major, lm_done)) = last_major_task {
                    if lm_done && !done {
                        tasks.push(WARN.to_owned() + " " + last_major);
                        last_major_task = None;
                    }
                }
            } else {
                last_major_task = Some((t.clone(), *done));
            }
            if !done {
                tasks.push(t.clone());
            }
        }

        tasks
    }

    pub fn list(&mut self, listall: bool) {
        let (left, dones) = self._get_all();

        if listall && !dones.is_empty() {
            for t in dones {
                println!("{}  {}", S_checked!(CHECKED), t.strikethrough())
            }
            println!();
        }

        if left.is_empty() {
            println!(" {} left!", S_empty!("nothing"));
        } else {
            let mut msg;
            let mut last_major_task :Option<(String, bool)> = None;
            let mut last_is_sub = false;

            for (t, done) in &self.tasks {
                msg = format!("{}  ", S_blink!(S_checkbox!(CHECKBOX)));
                if t.starts_with(SUB_PREFIX) {
                    if *done { continue }

                    msg = format!("{} {}", S_blink!(SUBTASK), msg);
                    msg += t.strip_prefix(SUB_PREFIX).unwrap();
                    last_is_sub = true;

                    if let Some((ref last_major, lm_done)) = last_major_task {
                        if lm_done {
                            println!("{} {} {}", S_checked!(CHECKED), WARN, last_major.strikethrough().bright_black());
                            last_major_task = None;
                        }
                    }
                } else {
                    last_major_task = Some((t.clone(), *done));

                    if *done { continue }

                    if last_is_sub {
                        last_is_sub = false;
                        msg = "\n".to_owned() + &msg;
                    }
                    msg = msg + t;
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
            println!("{} is not a valid source", S_moveto!("today"));
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
        #[allow(clippy::redundant_closure)]
        let mdfile= file.unwrap_or_else(|| util::pick_file());

        let fpath = Path::new(&mdfile);
        if ! fpath.is_file() {
            eprintln!("not a file or not exists: {}", S_fpath!(mdfile)); 
            std::process::exit(1)
        }
        println!("importing {} {}", S_fpath!(mdfile), PROGRESS);

        let mut newt = Vec::new();
        for rline in fs::read_to_string(fpath).expect("cannot read file").lines() {
            let line = rline.trim();
            if line.is_empty() { continue }

            if let Some(stripped) = line.strip_prefix(PREFIX) {
                println!("  {} : {}", S_checkbox!(CHECKBOX), stripped);
                newt.push((stripped.to_string(), false))
            }
        }

        if newt.is_empty() {
            println!("{} found!", S_empty!("nothing"));
        } else {
            self._load();
            self.tasks.append(&mut newt);
            self._dump();
        }
    }

    pub fn list_boxes(inbox_path: PathBuf) {
        let basedir = inbox_path.as_path().parent().unwrap();
        println!("[ {} ]", S_fpath!(basedir.display()));

        let mut boxes = Vec::new();
        for entry in fs::read_dir(basedir).expect("cannot read dir") {
            let path = entry.expect("cannot get entry").path();
            if path.is_file() && path.extension().unwrap() == "md" {
                boxes.push(String::from(path.file_stem().unwrap().to_str().unwrap()))
            }
        }
        boxes.sort(); boxes.reverse(); boxes.into_iter().for_each(
            |b| {
                print!("{}  {}",S_checkbox!(TASKBOX), b);
                let tbox = TaskBox::new(basedir.join(b).with_extension("md"));
                if tbox.alias.is_some() {
                    println!(" ({})", S_hints!(tbox.alias.unwrap()))
                } else {
                    println!()
                }
            })
    }
}
