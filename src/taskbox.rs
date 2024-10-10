use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use regex::Regex;
use colored::Colorize;
use lazy_static::lazy_static;

use crate::cli::*;
use crate::util::*;
use crate::styles::*;
use crate::conf::*;

lazy_static! {
    static ref RE_PREFIX_OPEN :Regex = Regex::new(r"^- \[[ ]\] (.*)").unwrap();
    static ref RE_PREFIX_DONE :Regex = Regex::new(r"^- \[[xX\-/<>\*]\] (.*)").unwrap();
    static ref RE_ROUTINES :Regex =
        Regex::new(r"\{󰃯:([dDwWbBqQmM]) (\d{4}-\d{2}-\d{2})\w{3} 󰳟\} (.*)").unwrap();
    static ref RE_ROUTINES_CHECKOUT :Regex =
        Regex::new(r"\{󰃯:(daily|weekly|biweekly|qweekly|monthly)\} (.*)").unwrap();
}

pub const INBOX_NAME :&str  = "INBOX";
pub const ROUTINE_BOXNAME :&str  = "ROUTINES";

const PREFIX_OPEN :&str  = "- [ ] ";
const PREFIX_DONE :&str  = "- [x] ";
const PREFIX_SUBT :&str  = " 󱞩 ";

#[derive(Debug)]
pub struct TaskBox {
    pub fpath: PathBuf,
    pub title: Option<String>,
    pub alias: Option<String>,
    pub tasks: Vec<(String, bool)>,
    pub selected: Option<Vec<String>>,
}

impl TaskBox {
    pub fn new(fpath: PathBuf) -> Self {
        Self {
            fpath,
            title: None, // None means not loaded
            alias: None,
            tasks: vec![],
            selected: None,
        }
    }

    pub fn sibling(&self, boxname: &str) -> Self {
        let mut sib = TaskBox::new(self.fpath.parent().unwrap()
            .join(get_box_unalias(boxname)).with_extension("md"));
        sib.load();
        sib
    }

    pub fn load(&mut self) {
        if self.title.is_some() { return } // avoid load() twice

        if ! self.fpath.exists() {
            let fpath = &self.fpath;
            let title = fpath.file_stem()
                             .and_then(|s| s.to_str())
                             .unwrap()
                             .to_string();

            fs::create_dir_all(fpath.parent().unwrap()).expect("Failed to create basedir");
            fs::File::create(fpath).expect("Failed to create file");
            fs::write(fpath, format!("# {}\n\n", title)).expect("Failed to write to file");

            // if it's "today" box, run 'checkout' once [only Unix]
            #[cfg(unix)]
            if title == get_today() || title == get_tomorrow() {
                use stdio_override::{StdoutOverride, StderrOverride};
                let null = "/dev/null";
                let  guard = StdoutOverride::override_file(null).unwrap();
                let eguard = StderrOverride::override_file(null).unwrap();

                self.collect_from(&mut self.sibling(ROUTINE_BOXNAME));

                drop(guard); drop(eguard);
            }
        }

        let mut raw_names = HashSet::new();
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
                    if raw_names.contains(&caps[1]) {
                        // hack way to avoid duplicate task name troubles
                        let mut newname = caps[1].to_string() + " ";
                        while raw_names.contains(&newname) {
                            // for multiple times duplicating
                            newname.push(' ')
                        }
                        raw_names.insert(String::from(&newname));
                        tasks.push((newname, false));
                    } else {
                        raw_names.insert(String::from(&caps[1]));
                        tasks.push((caps[1].to_string(), false))
                    }
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

        self.alias = get_box_alias(&title);
        self.title = Some(title);
        self.tasks = tasks;
    }

    fn _dump(&mut self) {
        let mut content = format!("# {}\n\n", self.title.clone().unwrap());

        for (mut task, done) in self.tasks.clone() {
            task = task.trim_end().to_string();

            if let Some(left) = task.strip_prefix(PREFIX_SUBT) {
                content.push_str("  ");
                task = left.to_string();
            }

            if done { content.push_str(PREFIX_DONE) }
            else {    content.push_str(PREFIX_OPEN) }
            content.push_str(&(task + "\n"))
        }

        fs::write(&self.fpath, content).expect("cannot write file")
    }

    // mark the task which has "done" subtask as "done"
    // return whether happened
    fn _mark_task_with_done_subtask(&mut self, task: &str) -> bool {
        if task.starts_with(PREFIX_SUBT) { return false }

        let mut found = false;
        let mut task_status :Option<&mut bool> = None;
        for (t, done) in self.tasks.iter_mut() {
            if ! found {
                if t == task && ! *done {
                    found = true;
                    task_status = Some(done);
                }
                continue

            } else if ! t.starts_with(PREFIX_SUBT) {
                return false
            } else if *done {
                // found done sub-task for this major task
                if task_status.is_some() {
                    *task_status.unwrap() = true;
                }
                return true
            }
        }
        false
    }

    fn _move_one(&mut self, from: &mut TaskBox, task: &str) {
        // just append it without dup checking, on purpuse
        self.tasks.push((task.to_string(), false));

        if ! from._mark_task_with_done_subtask(task) {
            // remove the one from "from"
            #[allow(clippy::nonminimal_bool)]
            from.tasks.retain(|(t, d)| ! (t == task && !d));
        }
    }

    fn _addone(&mut self, task: String) {
        let pair = (task, false);
        if ! self.tasks.contains(&pair) {
            self.tasks.push(pair)
        }
    }

    fn add_tasks(&mut self, tasks: Vec<String>) {
        if tasks.is_empty() { return }

        tasks.iter().for_each(|t| self._addone(t.to_string()));
        self._dump()
    }

    pub fn collect_from(&mut self, tb_from: &mut TaskBox) {
        fn _get_nickname(fp: &Path) -> String {
            let title_from_fp = fp.file_stem()
                                  .and_then(|s| s.to_str())
                                  .unwrap()
                                  .to_string();
            get_box_alias(&title_from_fp).unwrap_or(title_from_fp)
        }

        let tasks_in = tb_from.get_all_to_mark();
        if tasks_in.is_empty() { return }

        if let Some(ref selected) = tb_from.selected {
            if selected.is_empty() { return }
        }

        // print title line
        let from = _get_nickname(&tb_from.fpath);
        let to = _get_nickname(&self.fpath);
        println!("{} {} {} {}", S_movefrom!(from), MOVING,
                                S_moveto!(to), PROGRESS);

        // postpone self.load() to avoid stdio chaos(from daily hook)
        self.load();

        for task in tasks_in {
            if let Some(ref selected) = tb_from.selected {
                if ! selected.contains(&task) { continue }
            }

            let caps = RE_ROUTINES.captures(&task);

            if from == ROUTINE_BOXNAME {
                // non-routine tasks in routine box will be skipped
                // only "collect --inbox routines" (routines -> today/tomo) is valid
                if to != "today" && to != "tomorrow" { continue }

                if let Some(caps) = caps {
                    if ! util::match_routine(&caps[1], &caps[2], &to) {continue}

                    let kind = match &caps[1] {
                        "d" => "daily",
                        "w" => "weekly",
                        "b" => "biweekly",
                        "q" => "qweekly",
                        "m" => "monthly",
                        _ => "unknown",
                    };
                    let checkout_date = match to.as_ref() {
                        "today" => get_today(),
                        "tomorrow" => get_tomorrow(),
                        _ => panic!("unsupported checkout date(only today/tomorrow)"),
                    };
                    let newtask = format!("{{{}:{}}} {} [{} {}]",
                                           ROUTINES, kind, &caps[3], DATESTAMP, checkout_date);

                    println!("  {} : {}", S_checkbox!(ROUTINES), newtask);

                    let pair = (newtask, false);
                    if ! self.tasks.contains(&pair) {
                        self.tasks.push(pair)
                    }
                } else {
                    // ignore non-routine task
                    println!("{} {} : {} {}",
                            S_failure!(WARN),
                            S_checkbox!(CHECKBOX),
                            S_warning!("skip:"),
                            task);
                    continue
                }

            } else {

                if task.contains(WARN) {
                    println!("  {} : {}", S_checkbox!(CHECKED), task);
                } else if caps.is_some() {
                    println!("{} {} : {}",
                            S_failure!(WARN),
                            S_checkbox!(CHECKBOX),
                            task);
                } else if RE_ROUTINES_CHECKOUT.is_match(&task) && to == INBOX_NAME {
                    // ignore checkout routine task
                    println!("{} {} : {} {}",
                            S_failure!(WARN),
                            S_checkbox!(CHECKBOX),
                            S_warning!("skip:"),
                            task);
                    continue

                } else {
                    println!("  {} : {}", S_checkbox!(CHECKBOX), task);
                }

                self._move_one(tb_from, &task);
            }
        }

        // "ROUTINES" not drain
        if from != ROUTINE_BOXNAME {
            tb_from._dump();
        }
        self._dump();
    }

    pub fn add(&mut self, what: String,
                          routine: Option<Routine>,
                          add_date: bool,
                          start_date: &str) {
        self.load();

        let task = if let Some(routine) = routine {
            format!("{{{}:{} {}{} 󰳟}} {}",
                ROUTINES, 
                match routine {
                    Routine::Daily    => "d",
                    Routine::Weekly   => "w",
                    Routine::Biweekly => "b",
                    Routine::Qweekly  => "q",
                    Routine::Monthly  => "m",
                },
                start_date,
                weekday_from_date(start_date),
                what)

        } else if add_date {
            format!("{} [{} {}]", what, DATESTAMP, get_today())
        } else { what };

        self._addone(task);
        self._dump()
    }

    pub fn get_all_to_mark(&mut self) -> Vec<String> {
        self.load();

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
        self.load();
        let left : Vec<_> = self.tasks.iter().filter(|(_,done)| !done).map(|(task, _)| task.clone()).collect();
        let dones : Vec<_> = self.tasks.iter().filter(|(_,done)| *done).map(|(task, _)| task.clone()).collect();

        let checkbox_style = if self.title == Some("ROUTINES".into()) {
            ROUTINES
        } else {
            CHECKBOX
        };

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
                msg = format!("{}  ", S_blink!(S_checkbox!(checkbox_style)));
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

    pub fn count(&mut self) -> usize {
        self.load();
        self.tasks.iter().filter(|(_, done)| !done).count()
    }

    pub fn mark(&mut self, items: Vec<String>, delete: bool) {
        self.load();

        if items.is_empty() || self.tasks.is_empty() {
            return
        }

        for (task, done) in self.tasks.iter_mut() {
            if *done { continue }
            if items.contains(task) {
                *done = true;
            }
        }

        if delete {
            self.tasks.retain(|(task, done)| !done || !items.contains(task))
        }

        self._dump();
    }

    pub fn purge(&mut self, sort: bool) {
        self.load();
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
        let mut newr = Vec::new();
        for rline in fs::read_to_string(fpath).expect("cannot read file").lines() {
            let line = rline.trim();
            if line.is_empty() { continue }

            if let Some(stripped) = line.strip_prefix(PREFIX_OPEN) {
                if RE_ROUTINES.is_match(stripped) {
                    println!("  {} : {}", S_checkbox!(ROUTINES), stripped);
                    newr.push(stripped.to_string())
                } else {
                    println!("  {} : {}", S_checkbox!(CHECKBOX), stripped);
                    newt.push(stripped.to_string())
                }
            }
        }

        if newt.is_empty() && newr.is_empty() {
            println!("{} found!", S_empty!("nothing"));
            return
        }

        self.add_tasks(newt);
        self.sibling(ROUTINE_BOXNAME).add_tasks(newr);
    }
}
