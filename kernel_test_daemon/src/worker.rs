#![allow(dead_code)]
#![allow(unused_variables)]

use service::task_service_client::{TaskServiceClient};
use service::{TaskResult, FetchTaskRequest, UpdateResultRequest};
use tokio::time::{sleep, Duration};
use log::{error, warn, info, debug};
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
    let mut taskmgr = TaskMgr::new(&conf.get().rpc.taskcache.to_string(),
                                    &conf.get().execute.runner.to_string());
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
                warn!("Fetch Task Error {}", error);
            
                sleep(Duration::from_secs(3600)).await;
                continue;
            }
            let response = response.unwrap();
            let task = &response.get_ref().task;

            let task_id = task.task_id.to_owned();
            let command = task.command.to_owned();
            let args = task.args.clone();

            debug!("New Task Fetched. {} {} {}", &task_id, &command, match &args {
                None => {"None"},
                Some(val) => {val},
            });

            let task = task::worker::Task {
                state: "NEW".to_string(),
                task_id: task_id,
                command: command,
                args: args,
            };

            taskmgr.set_curr_task(task);
            taskmgr.store_on_disk();
        }

        info!("Execute task.");

        // execute & get the result
        let output = taskmgr.execute_curr_task().expect("execute task failed.");

        let code = output.status.code().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        
        info!("Task Done {} {} {}", code, &stdout, &stderr);
        
        let req_update_task = tonic::Request::new(UpdateResultRequest{
            task_result: TaskResult {
                task_id: taskmgr.get_curr_task().as_ref().unwrap().task_id.to_owned(),
                result: code,
                detail: match output.status.success() {
                        true => {Some(stdout)},
                        false => {Some(stderr)},
                    },
            }
        });



        let rs = client.update_result(req_update_task).await;
        if rs.is_ok() {
            taskmgr.clear_curr_task();
            taskmgr.store_on_disk();

            info!("Result updated to controller");
        } else if let Err(e) = rs {
            warn!("Result failed to update {}", e);
            
            sleep(Duration::from_secs(600)).await;
            continue;
        }

        sleep(Duration::from_secs(1800)).await;
    }
}