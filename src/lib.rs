use std::path::PathBuf;
use std::fs;
use std::io::Write;


#[derive(Debug)]
pub struct TaskBox {
    fpath: PathBuf,
    title: String,
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
            title: title,
            tasks: Vec::new(),
        }
    }
    fn _load(&mut self) {
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

        self.title = title;
        self.tasks = tasks;
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

    pub fn mark(&mut self, tasks: Vec<String>) {
        if tasks.is_empty() {
            return
        }

        let mut content = fs::read_to_string(&self.fpath).expect("Failed to read file");

        let orig_content = content.clone();
        let mut new_content = String::new();

        for task in tasks {
            if ! new_content.is_empty() {
                content = new_content.clone();
            }

            new_content = content
                .lines()
                .map(|line| {
                    if line.trim().starts_with("- [ ] ") && line.trim()[6..].eq(&task) {
                        line.replace("- [ ]", "- [x]")
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            new_content.push_str("\n")
        }
        
        if !new_content.is_empty() && new_content != orig_content {
            fs::write(&self.fpath, new_content).expect("cannot write file")
        }

        // refresh
        self._load();
    }
}
