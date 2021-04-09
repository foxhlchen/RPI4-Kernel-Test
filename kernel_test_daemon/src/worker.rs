use service::task_service_client::{TaskServiceClient};
use service::{Task, FetchTaskRequest, FetchTaskResponse, UpdateResultRequest, UpdateResultResponse};
use tokio::time::{sleep, Duration};
use log::{error, warn, info, debug, trace};
use log4rs;

pub mod service {
    tonic::include_proto!("service");
}

mod cfg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Load config");
    let conf = cfg::ConfigMgr::new().unwrap();

    info!("Initialize Log");
    log4rs::init_file(&conf.get().log.conf_path, Default::default()).unwrap();

    info!("Initialize RPC");
    let addr = conf.get().rpc.addr.to_string();

    loop {
        let mut client = TaskServiceClient::connect(addr.to_string()).await;
        if let Err(error) = client {
            error!("Connect Server Error {}", error);
            
            sleep(Duration::from_secs(3600)).await;
            continue;
        }

    }

    Ok(())

}