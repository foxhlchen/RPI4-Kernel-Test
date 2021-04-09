
mod cfg;
mod mail;
mod task;

use tonic::{transport::Server, Request, Response, Status};
use service::task_service_server::{TaskService, TaskServiceServer};
use service::{Task, FetchTaskRequest, FetchTaskResponse, UpdateResultRequest, UpdateResultResponse};
use log::{error, warn, info, debug, trace};
use chrono::prelude::*;
use log4rs;

pub mod service {
      tonic::include_proto!("service");
}

#[derive(Debug, Default)]
pub struct RealTaskService {}

#[tonic::async_trait]
impl TaskService for RealTaskService {
    async fn fetch_task(
        &self,
        _: Request<FetchTaskRequest>,
    ) -> Result<tonic::Response<FetchTaskResponse>, tonic::Status> {
        let mut rt = Err(Status::not_found("No Task found"));
        let mut tasks = task::TASKS.lock().unwrap();

        trace!("new FetchTaskRequest, pending task cnt: {}", tasks.len());
        let mut tasks_to_remove = Vec::new();
        for (seq, task) in tasks.iter() {
            let task_id = task.task_id.clone();
            let command = task.task_info.get("X-KernelTest-Branch").unwrap().clone();
            let deadline = task.get_deadline();

            if task.is_expired() {
                warn!("expired task {} deadline {}", &task_id, &deadline);
                tasks_to_remove.push(seq.clone());

                continue;
            }

            debug!("reply new task {} to worker", &task_id);
            let reply = FetchTaskResponse {task: Task {
                task_id: task_id,
                command: command,
                args: None,
            }};

            rt = Ok(Response::new(reply));
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
        let mut tasks = task::TASKS.lock().unwrap();
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

        tasks.remove(&seq);


        let reply = UpdateResultResponse {ret: 0};
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Load config");
    let conf = cfg::ConfigMgr::new().unwrap();

    info!("Initialize Log");
    log4rs::init_file(&conf.get().log.conf_path, Default::default()).unwrap();

    info!("Initialize RPC");
    let addr = conf.get().rpc.addr.parse()?;
    let taskservice = RealTaskService::default();

    info!("Start Task Manager");
    let taskmgr_handle = task::TaskMgr::start(conf)?;

    info!("Start RPC Service");
    let rpcserv = Server::builder()
        .add_service(TaskServiceServer::new(taskservice))
        .serve(addr);

    let rpcserv_handle = tokio::spawn( async move {
        rpcserv.await
    });

    let rs = tokio::try_join!(taskmgr_handle, rpcserv_handle);

    if let Err(error) = rs {
        error!("{}", &error);
    }

    Ok(())
}
