use log::{error};

pub struct MailMgr {
	session: imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
}

impl MailMgr {
	pub fn new(imap_conf: &super::cfg::IMap) -> imap::error::Result<Self> {
		let domain = &*imap_conf.domain;
		let tls = native_tls::TlsConnector::builder().build().unwrap();
		let client = imap::connect((domain, 993), domain, &tls).unwrap();
		
		let imap_session = client
			.login(&imap_conf.username, &imap_conf.password)
			.map_err(|e| e.0)?;

		let mut mailmgr = MailMgr {session: imap_session};

		mailmgr.select(&imap_conf.mailbox)?;

		Ok(mailmgr)
	}

	pub fn select(&mut self, mailbox: &str) -> imap::error::Result<imap::types::Mailbox> {
		self.session.select(mailbox)
	}

	pub fn list(&mut self) -> imap::error::Result<Vec<String>> {
		let list = self.session.list(None, Some("*"))?;
		let mut vec = Vec::new();

		for name in list.iter() {
			vec.push(name.name().to_string());
		}

		Ok(vec)
	}

	pub fn fetch_mail(&mut self, seq: u32)  -> imap::error::Result<String> {
		let messages = self.session.fetch(seq.to_string(), "RFC822")?;
		for message in messages.iter() {
			// extract the message's body
			let body = message.body();
			if let None = body {
				error!("mail {} has no body", seq);
			}
			let body = std::str::from_utf8(body.unwrap());
			if let Err(e) = body {
				error!("mail is not valid UTF8 {} {}", seq, e);
				continue
			}				
			let body = body.unwrap().to_string();
			return Ok(body);
		}
		Err(imap::Error::Bad("No mails found!".to_string()))
	}

	pub fn fetch_mails(
		&mut self, 
		seqs: std::collections::HashSet<u32>
	) -> imap::error::Result<Vec<(u32, String)>> {
		let mut vec = Vec::new();
		for seq in seqs {
			let messages = self.session.fetch(seq.to_string(), "RFC822")?;
			for message in messages.iter() {
				// extract the message's body
				let body = message.body();
				if let None = body {
					error!("mail {} has no body", seq);
				}
				let body = std::str::from_utf8(body.unwrap());
				if let Err(e) = body {
					error!("mail is not valid UTF8 {} {}", seq, e);
					continue
				}				
				let body = body.unwrap().to_string();
			
				vec.push((seq, body));
			}
		}
		
		Ok(vec)
	}

	pub fn fetch_unread(&mut self) -> imap::error::Result<std::collections::HashSet<u32>> {
		self.session.search("NOT SEEN")
	}
}

impl Drop for MailMgr {
	fn drop(&mut self) {
        let rs = self.session.logout();
		if rs.is_err() {
			let e = rs.unwrap_err();
			error!("{}", e.to_string());
		}
    }
}

pub fn list(imap_conf: &super::cfg::IMap) -> imap::error::Result<Vec<String>> {
	let domain = &*imap_conf.domain;
	let tls = native_tls::TlsConnector::builder().build().unwrap();
	let client = imap::connect((domain, 993), domain, &tls).unwrap();
    
	let mut imap_session = client
	    .login(&imap_conf.username, &imap_conf.password)
	    .map_err(|e| e.0)?;

	let list = imap_session.list(None, Some("*"))?;
	let mut vec = Vec::new();

	for name in list.iter() {
		vec.push(name.name().to_string());
	}

	imap_session.logout()?;

	Ok(vec)
}

pub fn fetch_unread(imap_conf: &super::cfg::IMap) -> imap::error::Result<Vec<(u32, String)>> {
	let domain = &*imap_conf.domain;
	let tls = native_tls::TlsConnector::builder().build().unwrap();
	let client = imap::connect((domain, 993), domain, &tls).unwrap();
	let mut imap_session = client
	    .login(&imap_conf.username, &imap_conf.password)
	    .map_err(|e| e.0)?;
    
	imap_session.select(&imap_conf.mailbox)?;
    
	// get unread messages
	let new_message_seqs = imap_session.search("NOT SEEN")?;
    
	//let size = new_message_seqs.len();
	//println!("new_message_seqs size {}", size);
    
	let mut vec = Vec::new();

	for seq in new_message_seqs {
	    let messages = imap_session.fetch(seq.to_string(), "RFC822")?;
	    println!("seq {}", seq);
	    for message in messages.iter() {
			// extract the message's body
			let body = message.body().expect("message did not have a body!");
			let body = std::str::from_utf8(body)
				.expect("message was not valid utf-8")
				.to_string();
		
			vec.push((seq, body));
	    }
	}
    
	imap_session.logout()?;
    
	Ok(vec)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_fetch() {
		let conf = crate::cfg::ConfigMgr::new().unwrap();
        let mut mailmgr = super::MailMgr::new(&conf.get().imap).unwrap();


		let seq = 97;
		let mail = mailmgr.fetch_mail(seq).unwrap();
		println!("{} {}", &seq, mail);

		/*for (seq, mail) in mails {
			println!("{} {}", seq, mail);
		}*/
    }

}
