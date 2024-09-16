use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use colored::Colorize;

use chrono::*;
use std::ops::*;

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
        if self.title.is_some() {
            return
        }


        let mut tasks = Vec::new();
        let mut title = String::new();

        for (index, rline) in fs::read_to_string(&self.fpath)
                            .expect("Failed to read file")
                            .lines().enumerate() {

            let line = rline.trim();
            if index == 0 {
                title = line.trim_start_matches("# ").to_string();

            } else if let Some(stripped) = line.strip_prefix(PREFIX) {
                tasks.push((stripped.to_string(), false))
            } else if let Some(stripped) = line.strip_prefix(PREFIX_DONE) {
                tasks.push((stripped.to_string(), true))
            }
        }

        self.title = Some(title);
        self.tasks = tasks;
    }

    fn _dump(&mut self) {
        let mut content = format!("# {}\n\n", self.title.clone().unwrap());

        for (task, done) in self.tasks.clone() {
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

        let (tasks, _) = todo_in._list();
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

        println!("{}  {} ↩️", from.unwrap().green(), to.unwrap().blue());

        for task in tasks {
            println!("{} : {}", " 󰄗".to_string().red(), task);
            self.tasks.push((task.clone(), false));
        }

        todo_in._clear();
        self._dump();
    }

    pub fn add(&mut self, what: String) {
        self._load();
        self.tasks.push((what, false));
        self._dump();
    }

    pub fn _list(&mut self) -> (Vec<String> ,Vec<String>) {
        self._load();
        (
            self.tasks.iter().filter(|(_,done)| !done).map(|(task, _)| task.clone()).collect(),
            self.tasks.iter().filter(|(_,done)| *done).map(|(task, _)| task.clone()).collect()
        )
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

    pub fn purge(&mut self) {
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
    pub fn collect(inbox_path: PathBuf) {
        let basedir = inbox_path.as_path().parent().unwrap();

        let mut today_todo = TaskBox::new(basedir.join(get_today()).with_extension("md"));
        let mut todo = TaskBox::new(inbox_path);
        today_todo._move_in(&mut todo)
    }

    // today -> INBOX
    pub fn postp(inbox_path: PathBuf) {
        let basedir = inbox_path.as_path().parent().unwrap();

        let mut today_todo = TaskBox::new(basedir.join(get_today()).with_extension("md"));
        let mut todo = TaskBox::new(inbox_path);
        todo._move_in(&mut today_todo)
    }

    // specified markdown file -> cur
    pub fn import(&mut self, mdfile: String) {
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
                println!("{} : {}", " 󰄗".to_string().red(), stripped);
                newt.push((stripped.to_string(), false))
            }
        }

        if newt.is_empty() {
            println!("{} found", "nothing".to_string().yellow())
        } else {
            self._load();
            self.tasks.append(&mut newt);
            self._dump();
        }
    }
}
