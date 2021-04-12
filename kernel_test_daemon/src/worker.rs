use service::task_service_client::{TaskServiceClient};
use service::{Task, FetchTaskRequest, FetchTaskResponse, UpdateResultRequest, UpdateResultResponse};
use tokio::time::{sleep, Duration};
use log::{error, warn, info, debug, trace};
use task::worker::*;

pub mod service {
    tonic::include_proto!("service");
}

mod cfg;
mod task;
mod mail;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Load config");
    let conf = cfg::worker::ConfigMgr::new().unwrap();

    info!("Initialize Log");
    log4rs::init_file(&conf.get().log.conf_path, Default::default()).unwrap();

    info!("Initialize RPC");
    let addr = conf.get().rpc.addr.to_string();

    info!("Initialize Task Manager");
    let mut taskmgr = TaskMgr::new(&conf.get().rpc.taskcache.to_string());
    taskmgr.load_from_disk();

    loop {
        let client = TaskServiceClient::connect(addr.to_string()).await;
        if let Err(error) = client {
            error!("Connect Server Error {}", error);
            
            sleep(Duration::from_secs(60)).await;
            continue;
        }
        let mut client = client.unwrap();

        // no task exists, fetch one
        if ! taskmgr.is_ongoing() {
            let req_newtask = tonic::Request::new(FetchTaskRequest{});
            let response = client.fetch_task(req_newtask).await;

            if let Err(error) = response {
                error!("Fetch Task Error {}", error);
            
                sleep(Duration::from_secs(60)).await;
                continue;
            }
            let response = response.unwrap();
            let task = &response.get_ref().task;

            let task_id = task.task_id.to_owned();
            let command = task.command.to_owned();
            let args = task.args.clone();

            let task = task::worker::Task {
                state: "NEW".to_string(),
                task_id: task_id,
                command: command,
                args: args,
            };

            taskmgr.set_curr_task(task);
        }

        // execute & get the result
        taskmgr.execute_curr_task();


    }

}