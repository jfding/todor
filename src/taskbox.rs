use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::*;
use std::ops::Add;

const PREFIX :&str  = "- [ ] ";
const PREFIX_DONE :&str  = "- [x] ";

#[derive(Debug)]
pub struct TaskBox {
    fpath: PathBuf,
    title: Option<String>,
    tasks: Vec<(String, bool)>,
}

impl TaskBox {
    pub fn new (fpath: PathBuf) -> Self {
        let title = fpath.file_stem().and_then(|s| s.to_str()).unwrap_or("TODO").to_string();

        if !fpath.exists() {
            fs::File::create(&fpath).expect("Failed to create file");
            fs::write(&fpath, format!("# {}\n\n", title)).expect("Failed to write to file");
        }
        
        Self {
            fpath: fpath,
            title: None, // None means not loaded
            tasks: Vec::new(),
        }
    }
    fn _load(&mut self) {
        if self.title != None {
            return
        }

        let content = fs::read_to_string(&self.fpath).expect("Failed to read file");
        
        let mut tasks = Vec::new();
        let mut title = String::new();

        for (index, line) in content.lines().enumerate() {
            if index == 0 {
                title = line.trim().trim_start_matches("# ").to_string();

            } else {
                let trimmed = line.trim();
                if trimmed.starts_with("- [") && trimmed.len() > 4 {
                    let completed = trimmed.chars().nth(3) == Some('x');
                    let task = trimmed[5..].trim().to_string();
                    tasks.push((task, completed));
                }
            }
        }

        self.title = Some(title);
        self.tasks = tasks;
    }

    fn _dump(&mut self) {
        let mut content = String::from(format!("# {}\n\n", self.title.clone().unwrap()));

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

    pub fn add(self, what: String) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.fpath)
            .expect("Failed to open file");

        writeln!(file, "- [ ] {}", what).expect("Failed to write to file");
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

    fn _move_in(&mut self, todo: &mut TaskBox) {
        self._load();

        todo._load();
        if todo.tasks.is_empty() { return }

        for (task, done) in todo.tasks.iter() {
            if ! *done {
                self.tasks.push((task.clone(), *done));
            }
        }
        todo._clear();

        self._dump();
    }

    pub fn sink(basedir: &Path) {
        use regex::Regex;
        let re = Regex::new(r"\d{4}-\d{2}-\d{2}.md$").unwrap();

        let today =  Local::now().date_naive();
        let mut today_todo = TaskBox::new(basedir.join(today.to_string()).with_extension("md"));

        let mut boxes = Vec::new();
        for entry in fs::read_dir(basedir).expect("cannot read dir") {
            let path = entry.expect("cannot get entry").path();
            if path.is_file() {
                if re.is_match(path.to_str().unwrap()) { 
                    boxes.push(path)
                }
            }
        }
        boxes.sort(); boxes.reverse();

        for taskbox in boxes {
            let boxdate = NaiveDate::parse_from_str(
                taskbox.file_stem().unwrap().to_str().unwrap(),
                "%Y-%m-%d").expect("something wrong!");

            if boxdate < today {
                let mut todo = TaskBox::new(taskbox);
                today_todo._move_in(&mut todo);
            } else {
                println!("future date {:?}", taskbox);
            }
        }
    }

    pub fn shift(basedir: &Path) {
        let today =  Local::now().date_naive();
        let tomor =  Local::now().add(chrono::Duration::days(1)).date_naive();
        let mut today_todo = TaskBox::new(basedir.join(today.to_string()).with_extension("md"));
        let mut tomor_todo = TaskBox::new(basedir.join(tomor.to_string()).with_extension("md"));
        tomor_todo._move_in(&mut today_todo)
    }
}
