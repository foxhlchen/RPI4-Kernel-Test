#![allow(dead_code)]
#![allow(unused_variables)]

use tonic::{transport::Server, Request, Response, Status};
use service::task_service_server::{TaskService, TaskServiceServer};
use service::{Task, FetchTaskRequest, FetchTaskResponse, UpdateResultRequest, UpdateResultResponse, Heartbeat};
use log::{error, warn, info, debug, trace};
use tokio::time::{sleep, Duration};

pub mod service {
      tonic::include_proto!("service");
}

mod cfg;
mod mail;
mod task;

#[derive(Debug, Default)]
pub struct RealTaskService {}

#[tonic::async_trait]
impl TaskService for RealTaskService {
    async fn fetch_task(
        &self,
        _: Request<FetchTaskRequest>,
    ) -> Result<tonic::Response<FetchTaskResponse>, tonic::Status> {
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
            if version.len() < 2 
                || version[0].parse::<i32>().unwrap() < 5 
                || version[1].parse::<i32>().unwrap() < 10 
            {
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
        let mut tasks = task::controller::TASKS.lock().unwrap();
        trace!("new UpdateResultRequest");

        let request = request.get_ref();
        let seq = request.task_result.task_id.parse::<u32>();
        if let Err(error) = seq {
            warn!("UpdateResultRequest task_id invalid {}", error);
            return Err(Status::not_found("No Task found"));
        }
        let seq = seq.unwrap();
        let task = tasks.get(&seq);
        if let None = task {
            warn!("UpdateResultRequest task_id not found {}", seq);
            return Err(Status::not_found("No Task found"));
        }
        let task = task.unwrap();

        if task.is_expired() {
            warn!("expired task {} deadline {}", &seq, &task.get_deadline());
        }

        task.reply_back(request.task_result.result, &request.task_result.detail);
        tasks.remove(&seq);

        let reply = UpdateResultResponse {ret: 0};
        Ok(Response::new(reply))
    }

    async fn heart_beat(
        &self,
        request: tonic::Request<Heartbeat>,
    ) -> Result<tonic::Response<Heartbeat>, tonic::Status> {
        Ok(Response::new(Heartbeat{}))
    }
}

async fn heartbeat() {
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Load config");
    let conf = cfg::controller::ConfigMgr::new().unwrap();

    info!("Initialize Log");
    log4rs::init_file(&conf.get().log.conf_path, Default::default()).unwrap();

    info!("Initialize RPC");
    let addr = conf.get().rpc.addr.parse()?;
    let taskservice = RealTaskService::default();

    info!("Start Task Manager");
    let taskmgr_handle = task::controller::TaskMgr::start(conf)?;

    info!("Start RPC Service");
    let rpcserv = Server::builder()
        .add_service(TaskServiceServer::new(taskservice))
        .serve(addr);

    let rpcserv_handle = tokio::spawn( async move {
        rpcserv.await
    });

    let heartbeat_handle = tokio::spawn(heartbeat());

    let rs = tokio::try_join!(taskmgr_handle, rpcserv_handle, heartbeat_handle);

    if let Err(error) = rs {
        error!("{}", &error);
    }

    Ok(())
}
