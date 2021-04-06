
mod cfg;
mod mail;
mod task;

use tonic::{transport::Server, Request, Response, Status};
use service::task_service_server::{TaskService, TaskServiceServer};
use service::{Task, FetchTaskRequest, FetchTaskResponse, UpdateResultRequest, UpdateResultResponse};
use log::{error, warn, info, debug, trace};
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
            let reply = FetchTaskResponse {task: Task {
                task_id: "".to_string(),
                command: "".to_string(),
                args: None,
            }};

            Ok(Response::new(reply))
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
    let taskmgr_handle = task::TaskMgr::start_new(conf)?;

    info!("Start RPC Service");
    let rpcserv = Server::builder()
        .add_service(TaskServiceServer::new(taskservice))
        .serve(addr);

    let rpcserv_handle = tokio::spawn( async move {
        rpcserv.await
    });

    let rs = tokio::try_join!(taskmgr_handle, rpcserv_handle);

    rs?;

    Ok(())
}
