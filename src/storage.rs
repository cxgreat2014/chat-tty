use std::path::PathBuf;

use directories::BaseDirs;
use rustyline::DefaultEditor;

pub struct Storage {
    pub rl: DefaultEditor,
    data_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Self {
        let data_dir = BaseDirs::new()
            .map(|s| s.data_dir().join("chat-tty"))
            .expect("Could not find OS base dir");
        Self {
            rl: DefaultEditor::new().expect("Unable to init rustyline"),
            data_dir,
        }
    }

    fn ensure_app_dir(&self) {
        let _ = std::fs::create_dir(&self.data_dir);
    }

    fn history_txt(&self) -> PathBuf {
        self.data_dir.join("history.txt")
    }

    pub fn load_history(&mut self) {
        self.ensure_app_dir();
        self.rl
            .load_history(&self.history_txt())
            .unwrap_or_else(|e| println!("{:?}", e));
    }

    pub fn write_history(&mut self) {
        self.ensure_app_dir();

        self.rl
            .save_history(&self.history_txt())
            .unwrap_or_else(|e| println!("{:?}", e));
    }
}
