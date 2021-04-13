use std::io::Write;
use std::fs::File;
use std::io::{self, BufRead};
use std::process::Command;
use log::{error, warn, info, debug, trace};

pub struct Task {
    pub state: String,
    pub task_id: String,
    pub command: String,
    pub args: Option<String>,
}

pub struct TaskMgr {
    current_task: Option<Task>,
    taskcache_path: String,
    runner_path: String,
}

impl TaskMgr {
    pub fn new(path: &str, runner: &str) -> Self {
        TaskMgr {current_task: None, taskcache_path: path.to_owned(), runner_path: runner.to_owned()}
    }

    pub fn load_from_disk(&mut self) {
        let file = File::open(&self.taskcache_path).unwrap();

        let mut task_info = Vec::new();
        for line in io::BufReader::new(file).lines() {
            if let Ok(val) = line {
                task_info.push(val);
            }
        }

        if task_info.len() < 3 {
            return;
        }

        let state = task_info[0].to_owned();
        let task_id = task_info[1].to_owned();
        let command = task_info[2].to_owned();
        let mut args = None;

        if task_info.len() >= 4 {
            args = Some(task_info[3].to_owned());
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

    pub fn execute_curr_task(&self) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        if self.current_task.is_none() {
            let err = std::boxed::Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "No Task Found"));

            return Err(err);
        }

        let task_state = self.current_task.as_ref().unwrap().state.to_owned();
        let task_id = self.current_task.as_ref().unwrap().task_id.to_owned();
        let task_command = self.current_task.as_ref().unwrap().command.to_owned();
        let args = self.current_task.as_ref().unwrap().args.clone();
        let runner_path = self.runner_path.to_owned();

        let mut program = Command::new(runner_path);
        let runner = program.arg(&task_state)
            .arg(&task_id).arg(&task_command);

        if let Some(args) = &args {
            runner.arg(args);
        }

        let rs = runner.output()?;


        Ok(rs)
    }
}
