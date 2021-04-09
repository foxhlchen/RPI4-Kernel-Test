
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
            request: tonic::Request<FetchTaskRequest>,
        ) -> Result<tonic::Response<FetchTaskResponse>, tonic::Status> {
            let mut tasks = task::TASKS.lock().unwrap();

            for (_, task) in tasks.iter() {
                let task_id = task.task_id.clone();
                let command = task.task_info.get("X-KernelTest-Branch").unwrap().clone();
                let deadline = task.task_info.get("X-KernelTest-Deadline").unwrap().clone();

                let rfc3339 = DateTime::parse_from_rfc3339(&deadline);
                if let Err(error) = rfc3339 {
                    error!("error deadline {} {} {}", &task_id, &deadline, error);
                    continue;
                }
                let deadline = rfc3339.unwrap();
                let now = Local::now();

                if now > deadline {
                    warn!("expired task {} deadline {} now {}", &task_id, &deadline, &now);
                    continue;
                }

                let reply = FetchTaskResponse {task: Task {
                    task_id: task_id,
                    command: command,
                    args: None,
                }};

                return Ok(Response::new(reply))
            }

            Err(tonic::Status::not_found("No Task found"))
        }

        async fn update_result(
            &self,
            request: tonic::Request<UpdateResultRequest>,
        ) -> Result<tonic::Response<UpdateResultResponse>, tonic::Status> {
            let reply = UpdateResultResponse {
                ret: 0,
            };

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
