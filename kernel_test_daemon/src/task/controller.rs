use log::{error, warn, info, debug, trace};
use tokio::time::{sleep, Duration};
use crate::mail::MailMgr;
use mailparse::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::{Mutex};
use lazy_static::lazy_static;
use std::io::Write;
use chrono::prelude::*;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::path::Path;

lazy_static! {
    pub static ref TASKS: Mutex<HashMap<u32, Task>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct Task {
    pub task_id: String,
    pub task_info: HashMap<String, String>,
}

pub struct TaskMgr {
    conf:  crate::cfg::controller::ConfigMgr,
}

fn format_deadline(deadline: &str) -> String {
    let mut s = String::new();
    let mut cnt = 0;
    for c in deadline.chars() {
        if c == '+' && cnt == 1 {
            s.push_str(":00");
        }

        if c == ':' {
            cnt += 1;
        }

        s.push(c);
    }

    s
}

impl Task {
    pub fn is_expired(&self) -> bool {
        let deadline = self.task_info.get("X-KernelTest-Deadline").unwrap().clone();

        let rfc3339 = DateTime::parse_from_rfc3339(&deadline);
        if let Err(error) = rfc3339 {
            error!("error deadline {} {} {}", &self.task_id, &deadline, error);
            
            return true;
        }
        let deadline = rfc3339.unwrap();
        let now = Local::now();

        now > deadline
    }

    pub fn get_deadline(&self) -> String {
        self.task_info.get("X-KernelTest-Deadline").unwrap().clone()
    }

    pub fn get_version(&self) -> String {
        self.task_info.get("X-KernelTest-Version").unwrap().clone()
    }

    pub fn get_branch(&self) -> String {        
        self.task_info.get("X-KernelTest-Branch").unwrap().clone()
    }

    pub fn is_valid_version(&self) -> bool {
        let version_rawstr = self.get_version();
        let version: Vec<&str> = version_rawstr.split('.').collect();
        if version.len() < 2 
            || version[0].parse::<i32>().unwrap() < 5 
            || version[1].parse::<i32>().unwrap() < 10 
        {            
            false
        } else {
            true
        }
    }

    pub fn reply_back(&self, result: i32, detail: &Option<String>) -> Result<(), lettre::transport::smtp::Error> {
        let cfgmgr = match crate::cfg::controller::ConfigMgr::new() {
            Ok(config) => config,
            Err(e) => panic!("{}", e)
        };

        let detail = match detail {
            Some(v) => v.to_owned(),
            None => "none".to_string(),
        };

        let version = self.get_version();

        let origin_subject = self.task_info.get("Subject").unwrap();
        let origin_body = self.task_info.get("Body").unwrap();
        let origin_from = self.task_info.get("From").unwrap();
        let origin_to = self.task_info.get("To").unwrap();
        let origin_date = self.task_info.get("Date").unwrap();
        let origin_msgid = self.task_info.get("Message-ID").unwrap();

        let subject = format!("RE: {}", origin_subject);
        let mut body = format!("On {}, {} wrote:\r\n", origin_date, origin_from);

        for origin_line in origin_body.lines() {
            if origin_line.starts_with("---") {
                break;
            }

            let newline = format!("> {}\r\n", origin_line);
            body.push_str(&newline);
        }

        let report = match result {
            0 => {
            format!(r#"
{} Compiled and booted on my Raspberry PI 4b (8g) (bcm2711)
            "#, &version);
            },
            _ => {
                format!(r#"
{} Failed to be compiled & booted on my Raspberry PI 4b (8g) (bcm2711)
Code: {}
Err:
{}
                "#, &version, &result, &detail)
            }
        };

        let signature = format!("\r\nTested-by: Fox Chen <foxhlchen@gmail.com>\r\n");

        let body = format!("{}{}{}", &body, &report, &signature);
        let from = cfgmgr.get().smtp.from.to_string();
        let to = cfgmgr.get().smtp.from.to_string();
        let in_reply_to = origin_msgid;

        trace!("compose email {} from {} to {} in_reply_to {} \n body: {} ", 
            &subject, &from, &to, &in_reply_to, &body);

        let email = Message::builder()
        .from(from.parse().unwrap())
        //.in_reply_to(in_reply_to.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(body)
        .unwrap();
    
        let creds = Credentials::new(cfgmgr.get().smtp.username.to_string(), cfgmgr.get().smtp.password.to_string());
    
        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay(&cfgmgr.get().smtp.domain)
            .unwrap()
            .credentials(creds)
            .build();
    
        // Send the email
        match mailer.send(&email) {
            Ok(_) => { 
                info!("{} {} Result Email sent successfully!", &self.task_id, &version); 
                Ok(())
            }
            Err(e) => { 
                info!("{} {} Result Email sending failed! {}", &self.task_id, &version, e);
                Err(e)
            }
        }
    }

    pub fn notify_worker_unresponded(
        from: &str, 
        to: &str, 
        username: &str, 
        passwd: &str, 
        domain: &str
    ) {
        let subject = format!("Worker Unresponded");
        let body = format!(r#"
Hi, 

The worker is unresponded for a while, please check to see what happened.


Thanks,
Testing bot
        "#, );
        let from = from.to_owned();
        let to = to.to_owned();
        let in_reply_to = "".to_string();

        trace!("compose email {} from {} to {} in_reply_to {} \n body: {} ", 
            &subject, &from, &to, &in_reply_to, &body);

        let email = Message::builder()
        .from(from.parse().unwrap())
        //.in_reply_to(_in_reply_to.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(body)
        .unwrap();
    
        let creds = Credentials::new(username.to_string(), passwd.to_string());
    
        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay(domain)
            .unwrap()
            .credentials(creds)
            .build();
    
        // Send the email
        match mailer.send(&email) {
            Ok(_) => { trace!("Notification Email sent successfully!") }
            Err(e) => { warn!("Notification Email sending failed!") }
        }
    }
}

impl TaskMgr {
    pub fn start(conf: crate::cfg::controller::ConfigMgr) -> Result<tokio::task::JoinHandle<()>, 
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
        let subject = headers.get_first_header("Subject");
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

        let body = parsed.get_body().unwrap();

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

        info_map.insert("Subject".to_owned(), subject);
        info_map.insert("Body".to_owned(), body);
        
        headers.get_first_value("Date").map(|v| {
            info_map.insert("Date".to_owned(), v);
        });

        headers.get_first_value("From").map(|v| {
            info_map.insert("From".to_owned(), v);
        });

        headers.get_first_value("To").map(|v| {
            info_map.insert("To".to_owned(), v);
        });

        headers.get_first_value("Message-ID").map(|v| {
            info_map.insert("Message-ID".to_owned(), v);
        });

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

        if ! info_map.contains_key("X-KernelTest-Deadline") {
            trace!("{} incorrect header", seq);

            return None;
        }

        let deadline = info_map.get("X-KernelTest-Deadline").unwrap().clone();
        let deadline = format_deadline(&deadline);
        info_map.insert("X-KernelTest-Deadline".to_owned(), deadline.clone());

        let task = Task {task_id: seq.to_string(), task_info: info_map};
        
        if task.is_expired() {
            warn!("expired task {} deadline {} now {}", &seq, &deadline, &Local::now());
            return None;
        }

        if !task.is_valid_version() {
            warn!("task {} version invalid {}", &seq, task.get_deadline());
            return None;
        }

        info!("new task from mail {} {} {}", seq, subject, &deadline);
        Some(task)
    }

    pub fn store_tasks_on_disk_raw(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;

        let ref tasks = *TASKS.lock().unwrap();
        for (seq, _task) in tasks {
            writeln!(&mut file, "{}", seq)?;
        }

        Ok(())
    }

    fn store_tasks_on_disk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = &self.conf.get().rpc.taskcache;
        Self::store_tasks_on_disk_raw(&self.conf.get().rpc.taskcache)
    }

    fn load_tasks_from_disk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(&self.conf.get().rpc.taskcache).exists() {
            info!("No task file on disk.");
            return Ok(())
        }

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
                    info!("[O] Loading task {} from disk succeeded.", seq);
                    TASKS.lock().unwrap().insert(seq, task);
                } else {
                    info!("[X] Loading task {} from disk failed.", seq);
                }
            }
        }

        Ok(())
    }

    async fn run(&mut self) {
        loop {
            let mailmgr = MailMgr::new(&self.conf.get().imap);
            if let Err(error) = mailmgr {
                error!("create mailmgr failed: {}", error);
                sleep(Duration::from_secs(600)).await; //back off for some time
                
                continue;
            }

            let mut mailmgr = mailmgr.unwrap();
            let fetch_rs = mailmgr.fetch_unread();
            if let Err(error) = fetch_rs {
                error!("fetch unread failed. error: {}", error);
                sleep(Duration::from_secs(600)).await; //back off for some time
                
                continue;
            }

            let unread_seqs = fetch_rs.unwrap();
            for seq in unread_seqs {
                let mail = mailmgr.fetch_mail(seq);
                if let Err(error) = mail {
                    error!("fetch mail {} failed. error: {}", seq, error);

                    continue;
                }

                let mail = mail.unwrap();
                let rs = Self::mail_to_task(seq, &mail);
                if let Some(task) = rs {
                    TASKS.lock().unwrap().insert(seq, task);
                }                           
            }
            
            match self.store_tasks_on_disk() {
                Ok(_) => debug!("storing tasks on disk finished"),
                Err(e) => error!("storing tasks on disk failed {}", e)                
            }

            info!("Polling unread mails done. waiting for next round.");
            sleep(Duration::from_secs(3600)).await; //sleep an hour
        }
    }
}
