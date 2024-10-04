use std::fs;
use tempfile::tempdir;
use todor::taskbox::*;
use todor::cli::*;
use todor::util::*;

fn setup_test_taskbox(name: &str) -> (TaskBox, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(name).with_extension("md");

    // test config settings
    let testtoml = dir.path().join("config.toml");
    let testcontent = format!("basedir = \"{}\"\nblink = false\n", dir.path().display());
    std::fs::write(&testtoml, testcontent).expect("write err");
    let test_conf = Config::load(Some(testtoml.to_str().unwrap().into()));

    let mut g_conf = CONFIG.write().unwrap();
    println!("{:?}", g_conf);
    println!("{:?}", test_conf);
    g_conf.update_with(&test_conf);

    (TaskBox::new(file_path), dir)
}

#[test]
fn test_taskbox_new() {
    let (tb, _dir) = setup_test_taskbox("test");
    assert_eq!(tb.title, None);
    assert_eq!(tb.alias, None);
    assert_eq!(tb.tasks.len(), 0);
}

#[test]
fn test_add_and_list() {
    let (mut tb, _dir) = setup_test_taskbox("test");
    tb.add("Test task".to_string(), None, false, "");
    tb.add("Test task with date".to_string(), None, true, "");

    tb.load();
    assert_eq!(tb.tasks.len(), 2);
    assert!(tb.tasks.contains(&("Test task".to_string(), false)));
    assert!(tb.tasks.iter().any(|(task, _)| task.starts_with("Test task with date")));
}

#[test]
fn test_mark() {
    let (mut tb, _dir) = setup_test_taskbox("test");
    tb.add("Task 1".to_string(), None, false, "");
    tb.add("Task 2".to_string(), None, false, "");
    tb.add("Task 3".to_string(), None, false, "");

    tb.mark(vec!["Task 1".to_string(), "Task 3".to_string()]);
    tb.load();
    assert_eq!(tb.tasks.iter().filter(|(_, done)| *done).count(), 2);
}

#[test]
fn test_purge() {
    let (mut tb, _dir) = setup_test_taskbox("test");
    tb.add("Task 1".to_string(), None, false, "");
    tb.add("Task 1".to_string(), None, false, "");
    tb.add("Task 3".to_string(), None, false, "");

    tb.purge(false);
    tb.load();
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
    tb1.load();
    assert_eq!(tb1.tasks.len(), 3);

    tb2.move_in(&mut tb1);

    tb2.load();
    assert_eq!(tb2.tasks.len(), 2);
    assert_eq!(tb2.tasks[0].0, "Task to move");
    assert_eq!(tb2.tasks[1].0, "Task2 to move");

    tb1.load();
    assert_eq!(tb1.tasks.len(), 1);
    assert_eq!(tb1.tasks[0].0, "Task not to move");
}

#[test]
fn test_move_in_with_warn_msg() {
    let (mut tb1, _dir1) = setup_test_taskbox("test1");
    let (mut tb2, _dir2) = setup_test_taskbox("test2");

    tb1.add("Task to move".to_string(), None, false, "");
    tb1.add("Daily routine".to_string(), Some(Routine::Daily), false, "");
    tb1.load();

    assert_eq!(tb1.tasks.len(), 2);

    tb2.move_in(&mut tb1);
    tb2.load();
    assert_eq!(tb2.tasks.len(), 2);
    assert_eq!(tb2.tasks[0].0, "Task to move");
    assert!(tb2.tasks[1].0.starts_with("{󰃯:d "));
    assert!(tb2.tasks[1].0.ends_with("} Daily routine"));

    tb1.load();
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
    tb1.load();
    assert_eq!(tb1.tasks.len(), 2);

    tb2.move_in(&mut tb1);

    let test1_actual = fs::read_to_string(&tb1.fpath).expect("Failed to read tb1 file");
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
    tb1.load();

    tb2.move_in(&mut tb1);

    let test1_actual = fs::read_to_string(&tb1.fpath).expect("Failed to read tb1 file");
    assert_eq!(test1_output, test1_actual);

    let test2_actual = fs::read_to_string(&tb2.fpath).expect("Failed to read tb2 file");
    assert_eq!(test2_output, test2_actual);
}

#[test]
fn test_move_in_with_dup_sub() {
    let (mut tb1, _dir1) = setup_test_taskbox("test1");
    let (mut tb2, _dir2) = setup_test_taskbox("test2");

    // Load prepared markdown files as test input
    let test1_input = r#"# test1

- [ ] Task to move
  - [ ] SubTask1 to move
  - [ ] SubTask1 to move
- [ ] Task2 to move
  - [x] SubTask1 to move
  - [ ] SubTask1 to move
- [ ] Task3 to move
  - [x] SubTask1 to move
  - [x] SubTask1 to move
- [x] Task4 to move
  - [ ] SubTask1 to move
  - [x] SubTask1 to move
"#;
    let test1_output = r#"# test1

- [x] Task2 to move
  - [x] SubTask1 to move
- [x] Task3 to move
  - [x] SubTask1 to move
  - [x] SubTask1 to move
- [x] Task4 to move
  - [x] SubTask1 to move
"#;
    let test2_output = r#"# test2

- [ ] Task to move
  - [ ] SubTask1 to move
  - [ ] SubTask1 to move
- [ ] Task2 to move
  - [ ] SubTask1 to move
- [ ] Task3 to move
- [ ] 󰼈 Task4 to move
  - [ ] SubTask1 to move
"#;

    std::fs::write(&tb1.fpath, test1_input).expect("Failed to write test input to file");
    tb1.load();

    tb2.move_in(&mut tb1);

    let test2_actual = fs::read_to_string(&tb2.fpath).expect("Failed to read tb2 file");
    assert_eq!(test2_output, test2_actual);

    // will failed, TODO to fix it with a design
    //let test1_actual = fs::read_to_string(&tb1.fpath).expect("Failed to read tb1 file");
    //assert_eq!(test1_output, test1_actual);
}

#[test]
fn test_add_routine() {
    let (mut tb, _dir) = setup_test_taskbox("test");
    tb.add("Daily routine".to_string(), Some(Routine::Daily), false, &get_today());

    tb.load();
    assert_eq!(tb.tasks.len(), 1);
    assert!(tb.tasks[0].0.starts_with("{󰃯:d "));
    assert!(tb.tasks[0].0.ends_with("} Daily routine"));
}

#[test]
fn test_checkout() {
    let (mut tb, _dir) = setup_test_taskbox("test");
    let (mut today, _dir) = setup_test_taskbox(&get_today());
    let (mut routine, _dir) = setup_test_taskbox(ROUTINE_BOXNAME);
    routine.add("Daily routine".to_string(), Some(Routine::Daily), false, &get_today());
    routine.add("ignore not routine".to_string(), None, false, "");

    routine.load();
    assert_eq!(routine.tasks.len(), 2);
    assert!(routine.tasks[0].0.starts_with("{󰃯:d "));
    assert!(routine.tasks[0].0.ends_with("} Daily routine"));

    today.move_in(&mut routine);

    today.load();
    assert_eq!(today.tasks.len(), 1);
    assert!(today.tasks[0].0.starts_with("{󰃯:daily} "));
    assert!(today.tasks[0].0.contains("} Daily routine"));
    assert!(today.tasks[0].0.contains(" [󰃵 "));

    tb.move_in(&mut routine);
    tb.load();
    assert_eq!(tb.tasks.len(), 0);

    today.move_in(&mut routine);
    today.load();
    assert_eq!(today.tasks.len(), 1);
}

#[test]
fn test_pool_today_to_inbox() {
    let today_input = format!(r#"# {}

- [ ] Task to move
"#, get_today());

    let (mut today, _dir) = setup_test_taskbox(&get_today());
    let (mut inbox, _dir) = setup_test_taskbox(INBOX_NAME);
    let (mut routine, _dir) = setup_test_taskbox(ROUTINE_BOXNAME);

    std::fs::write(&today.fpath, today_input).expect("Failed to write test input to file");
    today.load();

    today.add("Wrong daily routine".to_string(), Some(Routine::Daily), false, &get_today());
    inbox.add("old task".to_string(), None, false, "");
    routine.add("Daily routine".to_string(), Some(Routine::Daily), false, &get_today());

    today.load(); inbox.load(); routine.load();
    assert_eq!(today.tasks.len(), 2);
    assert_eq!(inbox.tasks.len(), 1);
    assert_eq!(routine.tasks.len(), 1);

    // check out
    today.move_in(&mut routine);

    today.load(); inbox.load(); routine.load();
    assert_eq!(today.tasks.len(), 3);
    assert_eq!(inbox.tasks.len(), 1);
    assert_eq!(routine.tasks.len(), 1);

    // pool
    inbox.move_in(&mut today);

    today.load(); inbox.load();
    assert_eq!(today.tasks.len(), 1);
    assert_eq!(inbox.tasks.len(), 3);
}

#[test]
fn test_import_somefile_to_inbox() {
    let md_input = r#"# free style file

- [ ] Task to import
        - [ ] Task to import with blank
## below one is a duplicated, will ingore
- [ ] Task to import
- [ ] Task2 to import
- [ ] {󰃯:d 2024-10-01} one daily to import
"#;

    let (mut inbox, dir) = setup_test_taskbox(INBOX_NAME);
    let mut routine = TaskBox::new(util::get_inbox_file("routine"));

    let fpath = dir.path().join("import-input").with_extension("md");
    std::fs::write(&fpath, md_input).expect("Failed to write test input to file");

    inbox.add("old task".to_string(), None, false, "");
    routine.add("old Daily routine".to_string(), Some(Routine::Daily), false, "");
    inbox.load(); routine.load();
    assert_eq!(inbox.tasks.len(), 1);
    assert_eq!(routine.tasks.len(), 1);

    routine = TaskBox::new(util::get_inbox_file("routine")); //reload
    inbox.import(Some(fpath.to_str().unwrap().to_string()));
    inbox.load(); routine.load();
    assert_eq!(inbox.tasks.len(), 4);
    //TODO assert_eq!(routine.tasks.len(), 2);
}
