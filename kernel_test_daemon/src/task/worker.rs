use std::fs::File;
use std::io::{self, BufRead};

pub struct Task {
    pub state: String,
    pub task_id: String,
    pub command: String,
    pub args: Option<String>,
}

pub struct TaskMgr {
    current_task: Option<Task>,
    taskcache_path: String,
}

impl TaskMgr {
    pub fn new(path: &str) -> Self {
        TaskMgr {current_task: None, taskcache_path: path.to_owned()}
    }

    pub fn load_from_disk(&mut self) {
        let file = File::open(&self.taskcache_path).unwrap();

        let task_info = Vec::new();
        for line in io::BufReader::new(file).lines() {
            if let Ok(val) = line {
                task_info.push(val);
            }
        }

        if task_info.len() < 3 {
            return;
        }

        let state = task_info[0];
        let task_id = task_info[1];
        let command = task_info[2];
        let mut args = None;

        if task_info.len() >= 4 {
            args = Some(task_info[3]);
        }

        self.current_task = Some(Task {
            state: state,
            task_id: task_id,
            command: command,
            args: args,
        });
    }

    pub fn store_on_disk(&self) {
        let mut file = File::create(&self.taskcache_path).unwrap();

        if let Some(task) = &self.current_task {
            writeln!(&mut file, "{}", &task.state).unwrap();
            writeln!(&mut file, "{}", &task.task_id).unwrap();
            writeln!(&mut file, "{}", &task.command).unwrap();
            if let Some(args) = &task.args {
                writeln!(&mut file, "{}", args).unwrap();
            }
        }
    }

    pub fn get_curr_task(&self) -> &Option<Task> {
        &self.current_task
    }

    pub fn is_ongoing(&self) -> bool {
        ! self.current_task.is_none()
    }

    pub fn set_curr_task(&mut self, task: Task) {
        self.current_task = Some(task);
    }

    pub fn execute_curr_task(&self) {

    }
}
