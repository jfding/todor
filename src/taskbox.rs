use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use colored::Colorize;
use chrono::*;
use lazy_static::lazy_static;

use crate::cli::*;
use crate::util::*;
use crate::styles::*;
use crate::conf::*;

lazy_static! {
    static ref RE_DATEBOX :Regex = Regex::new(r"\d{4}-\d{2}-\d{2}.md$").unwrap();
    static ref RE_PREFIX_OPEN :Regex = Regex::new(r"^- \[[ ]\] (.*)").unwrap();
    static ref RE_PREFIX_DONE :Regex = Regex::new(r"^- \[[xX\-/<>\*]\] (.*)").unwrap();
    static ref RE_ROUTINES    :Regex =
        Regex::new(r"\{󰃵:([dDwWbBqQmMoO]) (\d{4}-\d{2}-\d{2})\} (.*)").unwrap();
}

pub const INBOX_NAME :&str  = "INBOX";
pub const ROUTINE_BOXNAME :&str  = "ROUTINES";

const PREFIX_OPEN :&str  = "- [ ] ";
const PREFIX_DONE :&str  = "- [x] ";
const PREFIX_SUBT :&str  = " 󱞩 ";

#[derive(Debug)]
pub struct TaskBox {
    fpath: PathBuf,
    title: Option<String>,
    alias: Option<String>,
    tasks: Vec<(String, bool)>,
}

impl TaskBox {
    pub fn new (fpath: PathBuf) -> Self {
        let title = fpath.file_stem()
                         .and_then(|s| s.to_str())
                         .unwrap()
                         .to_string();

        if !fpath.exists() {
            fs::create_dir_all(fpath.parent().unwrap()).expect("Failed to create basedir");
            fs::File::create(&fpath).expect("Failed to create file");
            fs::write(&fpath, format!("# {}\n\n", title)).expect("Failed to write to file");
        }

        Self {
            fpath,
            title: None, // None means not loaded
            alias: get_box_alias(&title),
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
                if let Some(caps) = RE_PREFIX_OPEN.captures(line) {
                    tasks.push((caps[1].to_string(), false))
                } else if let Some(caps) = RE_PREFIX_DONE.captures(line) {
                    tasks.push((caps[1].to_string(), true))
                } else { continue }

                if last_is_sub {
                    last_is_sub = false;
                    postfix_sub += " "; // hack way to identify sub-tasks belong to diff task
                }
            } else {
                // might be sub-tasks
                let line = line.trim_start();

                if let Some(caps) = RE_PREFIX_OPEN.captures(line) {
                    tasks.push((PREFIX_SUBT.to_owned() + &caps[1] + &postfix_sub, false))
                } else if let Some(caps) = RE_PREFIX_DONE.captures(line) {
                    tasks.push((PREFIX_SUBT.to_owned() + &caps[1] + &postfix_sub, true))
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
            if let Some(left) = task.strip_prefix(PREFIX_SUBT) {
                content.push_str("  ");
                task = left.trim_end().to_string();
            }

            if done {
                content.push_str(PREFIX_DONE)
            } else {
                content.push_str(PREFIX_OPEN)
            }
            content.push_str(&(task + "\n"))
        }

        fs::write(&self.fpath, content).expect("cannot write file")
    }

    /// drain all uncompelted tasks
    fn _drain(&mut self) {
        self._load();

        // "ROUTINES" not drain
        if self.title == Some("ROUTINES".into()) { return }

        let mut newtasks = Vec::new();
        let mut last_major_task: Option<(String, bool)> = None;

        for (task, done) in self.tasks.iter() {
            if task.starts_with(PREFIX_SUBT) {
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

        println!("{} {} {} {}", S_movefrom!(from.unwrap()), MOVING,
                                S_moveto!(to.unwrap()), PROGRESS);

        for task in tasks {
            let pair = if task.contains(WARN) {
                println!("  {} : {}", S_checkbox!(CHECKED), task);
                (task.clone(), true)
            } else if let Some(caps) = RE_ROUTINES.captures(&task) {
                    if ! util::match_routine(&caps[1], &caps[2]) { continue }

                    let kind = match &caps[1] {
                        "d" => "daily",
                        "w" => "weekly",
                        "b" => "biweekly",
                        "q" => "qweekly",
                        "m" => "monthly",
                        _ => "unknown",
                    };
                    let newtask = format!("{} {{󰃵:{} by {}}}", &caps[3], kind, &caps[2]);

                    println!("  {} : {}", S_checkbox!(CALENDAR), newtask);
                    (newtask, false)
            } else {
                println!("  {} : {}", S_checkbox!(CHECKBOX), task);
                (task.clone(), false)
            };
            if ! self.tasks.contains(&pair) {
                self.tasks.push(pair)
            }
        }

        todo_in._drain();
        self._dump();
    }

    pub fn add(&mut self, what: String, routine: Option<Routine>, add_date: bool) {
        self._load();

        let task = match routine {
            Some(Routine::Daily)    => format!("{{󰃵:d {}}} {}", get_today(), what),
            Some(Routine::Weekly)   => format!("{{󰃵:w {}}} {}", get_today(), what),
            Some(Routine::Biweekly) => format!("{{󰃵:b {}}} {}", get_today(), what),
            Some(Routine::Qweekly)  => format!("{{󰃵:q {}}} {}", get_today(), what),
            Some(Routine::Monthly ) => format!("{{󰃵:m {}}} {}", get_today(), what),
            _ => {
                if add_date { format!("{} [󰃵 {}]", what, get_today()) }
                else { what }
            }
        };

        self.tasks.push((task, false));
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
            if t.starts_with(PREFIX_SUBT) {
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
                if t.starts_with(PREFIX_SUBT) {
                    if *done { continue }

                    msg = format!("{} {}", S_blink!(SUBTASK), msg);
                    msg += t.strip_prefix(PREFIX_SUBT).unwrap();
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

    pub fn count(&mut self) {
        self._load();
        let cc = self.tasks.iter().filter(|(_, done)| !done).count();
        if cc > 0 { println!("{}", cc) }
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
    pub fn sink(all: bool) {
        let basedir = Config_get!("basedir");
        let mut today_todo = TaskBox::new(util::get_inbox_file("today"));

        let mut boxes = Vec::new();
        for entry in fs::read_dir(basedir).expect("cannot read dir") {
            let path = entry.expect("cannot get entry").path();
            if path.is_file() && RE_DATEBOX.is_match(path.to_str().unwrap()) { 
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
                today_todo._move_in(&mut TaskBox::new(taskbox));
            }
        }
    }

    // today -> tomorrow
    pub fn shift() {
        TaskBox::new(util::get_inbox_file("tomorrow"))
            ._move_in(&mut
        TaskBox::new(util::get_inbox_file("today")))
    }

    // INBOX -> today
    pub fn collect(inbox_from: Option<String>) {
        let from = inbox_from.unwrap_or("inbox".into());

        if from == get_today() || from == "today" {
            println!("{} is not a valid source", S_moveto!("today"));
            return
        }

        TaskBox::new(util::get_inbox_file("today"))
            ._move_in(&mut
        TaskBox::new(util::get_inbox_file(&from)))
    }

    // today -> INBOX
    pub fn postp() {
        TaskBox::new(util::get_inbox_file("inbox"))
            ._move_in(&mut
        TaskBox::new(util::get_inbox_file("today")))
    }

    // specified markdown file -> cur
    pub fn import(&mut self, file: Option<String>) {
        #[allow(clippy::redundant_closure)]
        let mdfile= file.unwrap_or_else(|| super::util::pick_file());

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

            if let Some(stripped) = line.strip_prefix(PREFIX_OPEN) {
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
}
