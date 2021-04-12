pub struct Task {
    state: String,
}

pub struct TaskMgr {
    current_task: Option<Task>,
    taskcache_path: String,
}

impl TaskMgr {
    pub fn new(path: &str) -> Self {
        TaskMgr {current_task: None, taskcache_path: path.to_owned()}
    }

    pub fn load_from_disk(&self) {

    }

    pub fn store_on_disk(&self) {

    }

    pub fn get_curr_task(&self) -> &Option<Task> {
        &self.current_task
    }

    pub fn is_ongoing(&self) -> bool {
        ! self.current_task.is_none()
    }
}
