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
    static ref RE_ROUTINES :Regex =
        Regex::new(r"\{󰃯:([dDwWbBqQmM]) (\d{4}-\d{2}-\d{2})\} (.*)").unwrap();
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

    fn _addone(&mut self, task: String) {
        let pair = (task, false);
        if ! self.tasks.contains(&pair) {
            self.tasks.push(pair)
        }
    }
    fn _append1(&mut self, task: String) {
        self.tasks.push((task, false));
    }

    fn _move_one(&mut self, from: &mut TaskBox, task: &str) {
        // add new one to self
        self._append1(task.to_string());

        if ! from._mark_task_with_done_subtask(task) {
            // remove the one from "from"
            from.tasks.retain(|(t, _)| t != task)
        }
    }

    fn _move_in(&mut self, todo_from: &mut TaskBox) {
        self._load(); todo_from._load();

        let tasks = todo_from._get_all_to_mark();
        if tasks.is_empty() { return }

        // print title line
        let from = todo_from.alias.clone().unwrap_or(todo_from.title.clone().unwrap());
        let to = self.alias.clone().unwrap_or(self.title.clone().unwrap());
        println!("{} {} {} {}", S_movefrom!(from), MOVING,
                                S_moveto!(to), PROGRESS);

        for task in tasks {
            if task.contains(WARN) {
                println!("  {} : {}", S_checkbox!(CHECKED), task);
                self._move_one(todo_from, &task);
            } else if let Some(caps) = RE_ROUTINES.captures(&task) {
                if from != ROUTINE_BOXNAME || to != "today" {
                    // only "collect --inbox routines" (routines -> today) is valid
                    eprintln!("  {} : unexpected routine task move: {}",
                            S_failure!(WARN),
                            S_failure!(task));
                    self._move_one(todo_from, &task);
                } else {
                    if ! util::match_routine(&caps[1], &caps[2]) { continue }

                    let kind = match &caps[1] {
                        "d" => "daily",
                        "w" => "weekly",
                        "b" => "biweekly",
                        "q" => "qweekly",
                        "m" => "monthly",
                        _ => "unknown",
                    };
                    let newtask = format!("{{{}:{}}} {} [{} {}]", ROUTINES, kind, &caps[3], CALENDAR, get_today());

                    println!("  {} : {}", S_checkbox!(ROUTINES), newtask);
                    self._move_one(todo_from, &newtask);
                }
            } else {
                println!("  {} : {}", S_checkbox!(CHECKBOX), task);
                self._move_one(todo_from, &task);
            }
        }

        // "ROUTINES" not drain
        if from != ROUTINE_BOXNAME {
            todo_from._dump();
        }
        self._dump();
    }

    pub fn add(&mut self, what: String, routine: Option<Routine>, add_date: bool) {
        self._load();

        let task = if let Some(routine) = routine {
            format!("{{{}:{} {}}} {}",
                ROUTINES, 
                match routine {
                    Routine::Daily    => "d",
                    Routine::Weekly   => "w",
                    Routine::Biweekly => "b",
                    Routine::Qweekly  => "q",
                    Routine::Monthly  => "m",
                },
                get_today(), what)

        } else if add_date {
            format!("{} [{} {}]", what, CALENDAR, get_today())
        } else { what };

        self._addone(task);
        self._dump();
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
        self._load();
        let left : Vec<_> = self.tasks.iter().filter(|(_,done)| !done).map(|(task, _)| task.clone()).collect();
        let dones : Vec<_> = self.tasks.iter().filter(|(_,done)| *done).map(|(task, _)| task.clone()).collect();

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

        if ! newt.is_empty() {
            self._load();
            newt.iter().for_each(|t| self._addone(t.to_string()));
            self._dump();
        }
        if ! newr.is_empty() {
            let mut rbox = TaskBox::new(util::get_inbox_file("routine"));
            rbox._load();
            newr.iter().for_each(|t| rbox._addone(t.to_string()));
            rbox._dump();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_taskbox(name: &str) -> (TaskBox, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(name).with_extension("md");
        (TaskBox::new(file_path), dir)
    }

    #[test]
    fn test_taskbox_new() {
        let (tb, _dir) = setup_test_taskbox("test");
        assert!(tb.fpath.exists());
        assert_eq!(tb.title, None);
        assert_eq!(tb.tasks.len(), 0);
    }

    #[test]
    fn test_add_and_list() {
        let (mut tb, _dir) = setup_test_taskbox("test");
        tb.add("Test task".to_string(), None, false);
        tb.add("Test task with date".to_string(), None, true);

        tb._load();
        assert_eq!(tb.tasks.len(), 2);
        assert!(tb.tasks.contains(&("Test task".to_string(), false)));
        assert!(tb.tasks.iter().any(|(task, _)| task.starts_with("Test task with date")));
    }

    #[test]
    fn test_mark() {
        let (mut tb, _dir) = setup_test_taskbox("test");
        tb.add("Task 1".to_string(), None, false);
        tb.add("Task 2".to_string(), None, false);
        tb.add("Task 3".to_string(), None, false);

        tb.mark(vec!["Task 1".to_string(), "Task 3".to_string()]);
        tb._load();
        assert_eq!(tb.tasks.iter().filter(|(_, done)| *done).count(), 2);
    }

    #[test]
    fn test_purge() {
        let (mut tb, _dir) = setup_test_taskbox("test");
        tb.add("Task 1".to_string(), None, false);
        tb.add("Task 1".to_string(), None, false);
        tb.add("Task 3".to_string(), None, false);

        tb.purge(false);
        tb._load();
        assert_eq!(tb.tasks.len(), 2);
    }

    #[test]
    fn test_move_in_basic() {
        let (mut tb1, _dir1) = setup_test_taskbox("test1");
        let (mut tb2, _dir2) = setup_test_taskbox("test2");

        // Load prepared markdown files as test input
        let test1_input = r#"# test1

- [ ] Task to move
- [x] Task not to move
- [ ] Task2 to move
"#;
        std::fs::write(&tb1.fpath, test1_input).expect("Failed to write test input to file");
        tb1._load();
        assert_eq!(tb1.tasks.len(), 3);

        tb2._move_in(&mut tb1);

        tb2._load();
        assert_eq!(tb2.tasks.len(), 2);
        assert_eq!(tb2.tasks[0].0, "Task to move");
        assert_eq!(tb2.tasks[1].0, "Task2 to move");

        tb1._load();
        assert_eq!(tb1.tasks.len(), 1);
        assert_eq!(tb1.tasks[0].0, "Task not to move");
    }

    #[test]
    fn test_move_in_with_warn_msg() {
        let (mut tb1, _dir1) = setup_test_taskbox("test1");
        let (mut tb2, _dir2) = setup_test_taskbox("test2");

        tb1.add("Task to move".to_string(), None, false);
        tb1.add("Daily routine".to_string(), Some(Routine::Daily), false);
        tb1._load();

        assert_eq!(tb1.tasks.len(), 2);

        tb2._move_in(&mut tb1);
        tb2._load();
        assert_eq!(tb2.tasks.len(), 2);
        assert_eq!(tb2.tasks[0].0, "Task to move");
        assert!(tb2.tasks[1].0.starts_with("{󰃯:d "));
        assert!(tb2.tasks[1].0.ends_with("} Daily routine"));

        tb1._load();
        assert_eq!(tb1.tasks.len(), 0);
    }

    #[test]
    fn test_move_in_with_sub() {
        let (mut tb1, _dir1) = setup_test_taskbox("test1");
        let (mut tb2, _dir2) = setup_test_taskbox("test2");

        // Load prepared markdown files as test input
        let test1_input = r#"# test1

- [ ] Task to move
  - [ ] SubTask1 to move
"#;
        let test1_output = r#"# test1

"#;
        let test2_output = r#"# test2

- [ ] Task to move
  - [ ] SubTask1 to move
"#;

        std::fs::write(&tb1.fpath, test1_input).expect("Failed to write test input to file");
        tb1._load();
        assert_eq!(tb1.tasks.len(), 2);

        tb2._move_in(&mut tb1);

        let test1_actual = fs::read_to_string(&tb1.fpath).expect("Failed to read tb2 file");
        assert_eq!(test1_output, test1_actual);

        let test2_actual = fs::read_to_string(&tb2.fpath).expect("Failed to read tb2 file");
        assert_eq!(test2_output, test2_actual);
    }

    #[test]
    fn test_move_in_with_sub_done() {
        let (mut tb1, _dir1) = setup_test_taskbox("test1");
        let (mut tb2, _dir2) = setup_test_taskbox("test2");

        // Load prepared markdown files as test input
        let test1_input = r#"# test1

- [ ] Task to move but keep with "done" status
  - [x] SubTask1 NOT move
- [ ] Task2 to move
- [ ] Task3 to move but keep with "done" status
  - [ ] SubTask1 to move
  - [x] SubTask2 NOT move
- [x] Task4 NOT move
  - [x] SubTask1 NOT move
- [x] Task5 to move with warning icon
  - [ ] SubTask1 to move
  - [x] SubTask2 NOT move
  - [ ] SubTask3 to move
  "#;
        let test1_output = r#"# test1

- [x] Task to move but keep with "done" status
  - [x] SubTask1 NOT move
- [x] Task3 to move but keep with "done" status
  - [x] SubTask2 NOT move
- [x] Task4 NOT move
  - [x] SubTask1 NOT move
- [x] Task5 to move with warning icon
  - [x] SubTask2 NOT move
"#;
        let test2_output = r#"# test2

- [ ] Task to move but keep with "done" status
- [ ] Task2 to move
- [ ] Task3 to move but keep with "done" status
  - [ ] SubTask1 to move
- [ ] 󰼈 Task5 to move with warning icon
  - [ ] SubTask1 to move
  - [ ] SubTask3 to move
"#;

        std::fs::write(&tb1.fpath, test1_input).expect("Failed to write test input to file");
        tb1._load();

        tb2._move_in(&mut tb1);

        let test1_actual = fs::read_to_string(&tb1.fpath).expect("Failed to read tb2 file");
        assert_eq!(test1_output, test1_actual);

        let test2_actual = fs::read_to_string(&tb2.fpath).expect("Failed to read tb2 file");
        assert_eq!(test2_output, test2_actual);
    }

    #[test]
    fn test_add_routine() {
        let (mut tb, _dir) = setup_test_taskbox("test");
        tb.add("Daily routine".to_string(), Some(Routine::Daily), false);

        tb._load();
        assert_eq!(tb.tasks.len(), 1);
        assert!(tb.tasks[0].0.starts_with("{󰃯:d "));
        assert!(tb.tasks[0].0.ends_with("} Daily routine"));
    }
}
