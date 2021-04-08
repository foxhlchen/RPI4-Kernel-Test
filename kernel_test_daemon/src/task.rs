
use log::{error, warn, info, debug, trace};
use tokio::time::{sleep, Duration};
use crate::mail::MailMgr;
use mailparse::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use std::io::Write;

lazy_static! {
    pub static ref TASKS: Mutex<HashMap<u32, Task>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct Task {
    task_id: String,
    task_info: HashMap<String, String>,
}

pub struct TaskMgr {
    conf:  super::cfg::ConfigMgr,
}

impl TaskMgr {
    pub fn start(conf: super::cfg::ConfigMgr) -> Result<tokio::task::JoinHandle<()>, 
    Box<dyn std::error::Error>> {
        let mut newmgr = TaskMgr{conf};

        newmgr.init()?;

        Ok(tokio::spawn(async move {
            newmgr.run().await
        }))
    }

    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_tasks_from_disk()?;
        Ok(())
    }

    fn mail_to_task(seq: u32, email: &String) -> Option<Task> {
        // Parse mail
        let parsed_rs = parse_mail(email.as_bytes());
        if let Err(error) = parsed_rs {
            error!("parse mail {} failed. error: {}", seq, error);
            
            return None;
        }
        let parsed = parsed_rs.unwrap();
        let headers = parsed.get_headers();

        // Check Subject
        let subject = headers.get_first_header("From");
        if subject.is_none() {
            error!("parse mail {} failed. error: {}", seq, "No subject exists");
            
            return None;
        }
        let subject = subject.unwrap().get_value();
        if subject.len() <= 6 {
            trace!("{} incorrect subject {}", seq, subject);

            return None;
        }

        if &subject[..6] != "[PATCH" {
            trace!("{} incorrect subject {}", seq, subject);

            return None;
        }

        if &subject[subject.len() - 6..] != "review" {
            trace!("{} incorrect subject {}", seq, subject);

            return None;
        }

/*
X-stable: review
X-Patchwork-Hint: ignore
X-KernelTest-Patch: http://kernel.org/pub/linux/kernel/v5.x/stable-review/patch-5.11.12-rc1.gz
X-KernelTest-Tree: git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux-stable-rc.git
X-KernelTest-Branch: linux-5.11.y
X-KernelTest-Patches: git://git.kernel.org/pub/scm/linux/kernel/git/stable/stable-queue.git
X-KernelTest-Version: 5.11.12-rc1
X-KernelTest-Deadline: 2021-04-07T08:50+00:00
*/

        let mut info_map: HashMap<String, String> = HashMap::new();
        headers.get_first_value("X-stable").map(|v| {
            info_map.insert("X-stable".to_owned(), v);
        });

        headers.get_first_value("X-KernelTest-Patch").map(|v| {
            info_map.insert("X-KernelTest-Patch".to_owned(), v);
        });

        headers.get_first_value("X-KernelTest-Tree").map(|v| {
            info_map.insert("X-KernelTest-Tree".to_owned(), v);
        });

        headers.get_first_value("X-KernelTest-Branch").map(|v| {
            info_map.insert("X-KernelTest-Branch".to_owned(), v);
        });

        headers.get_first_value("X-KernelTest-Patches").map(|v| {
            info_map.insert("X-KernelTest-Patches".to_owned(), v);
        });

        headers.get_first_value("X-KernelTest-Version").map(|v| {
            info_map.insert("X-KernelTest-Version".to_owned(), v);
        });

        headers.get_first_value("X-KernelTest-Deadline").map(|v| {
            info_map.insert("X-KernelTest-Deadline".to_owned(), v);
        });

        if ! info_map.contains_key("X-KernelTest-Version") {
            trace!("{} incorrect header", seq);

            return None;
        }

        info!("new task from mail {} {}", seq, subject);
        Some(Task {task_id: seq.to_string(), task_info: info_map})
    }

    fn store_tasks_on_disk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(&self.conf.get().rpc.taskcache)?;

        let ref tasks = *TASKS.lock().unwrap();
        for (seq, task) in tasks {
            writeln!(&mut file, "{}", seq);
        }

        Ok(())
    }

    fn load_tasks_from_disk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(&self.conf.get().rpc.taskcache)?;
        let mut mailmgr = MailMgr::new(&self.conf.get().imap).unwrap();

        for line in io::BufReader::new(file).lines() {
            if let Ok(seq) = line {
                let seq = seq.parse::<u32>()?;

                let mail = mailmgr.fetch_mail(seq);
                if mail.is_err() {
                    let error = mail.unwrap_err();
                    error!("fetch mail {} failed. error: {}", seq, error.to_string());

                    continue;
                }
                let mail = mail.unwrap();
                let rs = Self::mail_to_task(seq, &mail);
                if let Some(task) = rs {
                    TASKS.lock().unwrap().insert(seq, task);
                }                           
            }
        }

        Ok(())
    }

    async fn run(&mut self) {
        loop {
            let mut mailmgr = MailMgr::new(&self.conf.get().imap).unwrap();
            let fetch_rs = mailmgr.fetch_unread();
            if let Err(error) = fetch_rs {
                error!("fetch unread failed. error: {}", error.to_string());
                sleep(Duration::from_secs(600)).await; //back off for some time
                
                continue;
            }

            let unread_seqs = fetch_rs.unwrap();
            for seq in unread_seqs {
                let mail = mailmgr.fetch_mail(seq);
                if let Err(error) = mail {
                    error!("fetch mail {} failed. error: {}", seq, error.to_string());

                    continue;
                }

                let mail = mail.unwrap();
                let rs = Self::mail_to_task(seq, &mail);
                if let Some(task) = rs {
                    TASKS.lock().unwrap().insert(seq, task);
                }                           
            }
            
            trace!("Polling unread mails done.");
            sleep(Duration::from_secs(3600)).await; //sleep an hour
        }
    }
}
