use log::{error, warn, info, debug, trace};


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

