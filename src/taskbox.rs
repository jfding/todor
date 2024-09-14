use std::fs;
use std::io::Write;
use std::path::PathBuf;

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

    pub fn list(&mut self, all: Option<bool>) -> Vec<String> {
        self._load();

        let all = all.unwrap_or(false); // Default value is false
        self.tasks.iter().filter(|(_,done)| all || !done).map(|(task, _)| task.clone()).collect()
    }

    pub fn count(mut self) -> usize {
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
        use cmd_lib::run_cmd;
        let fpath = self.fpath.display();
        run_cmd!(
            awk "!seen[$0]++" $fpath > /tmp/todor.tmp.md;
            mv /tmp/todor.tmp.md $fpath
        ).expect("cannot run awk")
    }
}
