
use log::{error, warn, info, debug, trace};
use tokio::time::{sleep, Duration};
use regex::Regex;

struct Task {
    task_id: String,
}

pub struct TaskMgr {
    tasks: Vec<Task>,
    conf:  super::cfg::ConfigMgr,
}

impl TaskMgr {
    pub fn start_new(conf: super::cfg::ConfigMgr) -> Result<tokio::task::JoinHandle<()>, 
    Box<dyn std::error::Error>> {
        let newmgr = TaskMgr{tasks: Vec::new(), conf};

        Ok(tokio::spawn(async move {
            newmgr.run();
        }))
    }

    fn init() -> Result<(), Box<dyn std::error::Error>> {
        

        Ok(())
    }
    
    async fn run(&self) {
        let re = Regex::new(r"^$").unwrap();

        loop {
            let fetch_rs = super::mail::fetch_unread(&self.conf.get().imap);
            if fetch_rs.is_err() {
                let Err(error) = fetch_rs;
                error!("fetch mail failed. error: {}", error.to_string());

                sleep(Duration::from_secs(3600)).await; //back off for an hour
                continue;
            }

            let Ok(mails) = fetch_rs;
            for (seq, mail) in mails {

                assert!(re.is_match("2014-01-01"));
            }
        }
    }
}