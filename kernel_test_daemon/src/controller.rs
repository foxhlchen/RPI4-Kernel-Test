#![allow(dead_code)]
#![allow(unused_variables)]

use tonic::{transport::Server, Request, Response, Status};
use service::task_service_server::{TaskService, TaskServiceServer};
use service::{Task, FetchTaskRequest, FetchTaskResponse, UpdateResultRequest, UpdateResultResponse, Heartbeat};
use log::{error, warn, info, debug, trace};
use tokio::time::{sleep, Duration};
use chrono::prelude::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

pub mod service {
      tonic::include_proto!("service");
}

mod cfg;
mod mail;
mod task;

lazy_static! {
    pub static ref UPDATE_TIME: Mutex<chrono::DateTime<chrono::Local>> = {
        Mutex::new(Local::now())
    };
}

#[derive(Debug, Default)]
pub struct RealTaskService {
    taskcache: String,
}

#[tonic::async_trait]
impl TaskService for RealTaskService {
    async fn fetch_task(
        &self,
        _: Request<FetchTaskRequest>,
    ) -> Result<tonic::Response<FetchTaskResponse>, tonic::Status> {
        {
            let mut guard = UPDATE_TIME.lock().unwrap();
            *guard = Local::now();
        }

        let mut rt = Err(Status::not_found("No Task found"));
        let mut tasks = task::controller::TASKS.lock().unwrap();

        trace!("new FetchTaskRequest, pending task cnt: {}", tasks.len());
        let mut tasks_to_remove = Vec::new();
        for (seq, task) in tasks.iter() {
            let task_id = task.task_id.clone();
            let command = task.get_version();
            let args = task.get_branch();
            let deadline = task.get_deadline();

            let version_rawstr = task.get_version();
            let version: Vec<&str> = version_rawstr.split('.').collect();
            if !task.is_valid_version() {
                warn!("invalid task {} version {}", &task_id, &version_rawstr);
                tasks_to_remove.push(seq.clone());

                continue;
            }

            if task.is_expired() {
                warn!("expired task {} deadline {}", &task_id, &deadline);
                tasks_to_remove.push(seq.clone());

                continue;
            }

            debug!("reply new task {} to worker", &task_id);
            let reply = FetchTaskResponse {task: Task {
                task_id: task_id,
                command: command,
                args: Some(args),
            }};

            rt = Ok(Response::new(reply));
            break;
        }

        for task_id in tasks_to_remove.iter() {
            trace!("remove task {}", task_id);
            tasks.remove(task_id);
        }

        rt
    }

    async fn update_result(
        &self,
        request: tonic::Request<UpdateResultRequest>,
    ) -> Result<tonic::Response<UpdateResultResponse>, tonic::Status> {
        let reply = UpdateResultResponse {ret: 0};
        {
            let mut guard = UPDATE_TIME.lock().unwrap();
            *guard = Local::now();
        }

        // with TASKS.lock
        {
            let mut tasks = task::controller::TASKS.lock().unwrap();
            trace!("new UpdateResultRequest");

            let request = request.get_ref();
            let seq = request.task_result.task_id.parse::<u32>();
            if let Err(error) = seq {
                warn!("UpdateResultRequest task_id invalid {}", error);
                return Ok(Response::new(reply));
            }
            let seq = seq.unwrap();
            let task = tasks.get(&seq);
            if let None = task {
                warn!("UpdateResultRequest task_id not found {}", seq);
                return Ok(Response::new(reply));
            }
            let task = task.unwrap();

            if task.is_expired() {
                warn!("expired task {} deadline {}", &seq, &task.get_deadline());
                return Ok(Response::new(reply));
            }

            if let Err(e) = task.reply_back(request.task_result.result, &request.task_result.detail) {
                warn!("Send back result error, task {} deadline {} error {}", &seq, &task.get_deadline(), e);
                return Err(Status::not_found("Failed to send result email, Please retry later."));
            }
            tasks.remove(&seq);
        }

        let rs = task::controller::TaskMgr::store_tasks_on_disk_raw(&self.taskcache);
        if let Err(error) = rs {
            warn!("Update taskcache file failed. {}", error);
        }

        Ok(Response::new(reply))
    }

    async fn heart_beat(
        &self,
        request: tonic::Request<Heartbeat>,
    ) -> Result<tonic::Response<Heartbeat>, tonic::Status> {
        {
            let mut guard = UPDATE_TIME.lock().unwrap();
            *guard = Local::now();
        }

        Ok(Response::new(Heartbeat{}))
    }
}

async fn heartbeat(from: String, to: String, username: String, passwd: String, domain: String) {
    loop {
        {
            let guard = UPDATE_TIME.lock().unwrap();
            let now = Local::now();
            let diff = guard.signed_duration_since(now);

            if diff.num_hours() >= 10 {
                error!("{} hours elapses, worker error", diff.num_hours());
                task::controller::Task::notify_worker_unresponded(&from, &to, &username, &passwd, &domain);
            }
        }
        sleep(Duration::from_secs(3600)).await;
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Load config");
    let conf = cfg::controller::ConfigMgr::new().unwrap();

    info!("Initialize Heartbeat handler");
    let from = conf.get().smtp.from.to_owned();
    let to = conf.get().smtp.from.to_owned();
    let username = conf.get().smtp.username.to_owned();
    let passwd = conf.get().smtp.password.to_owned();
    let domain = conf.get().smtp.domain.to_owned();

    let heartbeat_handle = tokio::spawn(heartbeat(from, to, username, passwd, domain));

    info!("Initialize Log");
    log4rs::init_file(&conf.get().log.conf_path, Default::default()).unwrap();

    info!("Initialize RPC");
    let addr = conf.get().rpc.addr.parse()?;
    let taskservice = RealTaskService {
        taskcache: conf.get().rpc.taskcache.to_owned(),
    };

    info!("Start Task Manager");
    let taskmgr_handle = task::controller::TaskMgr::start(conf)?;

    info!("Start RPC Service");
    let rpcserv = Server::builder()
        .add_service(TaskServiceServer::new(taskservice))
        .serve(addr);

    let rpcserv_handle = tokio::spawn( async move {
        rpcserv.await
    });

    let rs = tokio::try_join!(taskmgr_handle, rpcserv_handle, heartbeat_handle);

    if let Err(error) = rs {
        error!("{}", &error);
    }

    Ok(())
}
