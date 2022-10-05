use std::{io, sync::mpsc};
//use serde::{Serialize, Deserialize};

pub struct Message {
	pub user_key: Vec<u8>,
	pub mes: Vec<u8>,
	pub ip: Vec<u8>
}

impl Message {
	pub fn new(user_key: Vec<u8>, mes: Vec<u8>, ip: Vec<u8>) -> Self {
		Self { user_key, mes, ip }
	}
	pub fn as_bytes(&self) -> Vec<u8> {//&[u8] {
		let mut v: Vec<u8> = Vec::new();
		for e in &self.user_key {
			v.push(*e);
		}
		for l in &self.ip {
			v.push(*l);
		}
		for e in &self.mes {
			v.push(*e);
		}
		v
		//let bytes: [u8] = v.as_slice();
		//let bytes: &[u8] = &v[..];
		//bytes
	}

	pub fn print(&self) {
		println!("----Message----");
		match std::str::from_utf8(&self.mes) {
			Ok(s) => {println!("{}",s);},
			Err(_) => {}
		}
		println!("from\n- ");
		//let user: Vec<u8> = rsa.public_key_to_pem().unwrap();
		println!("{}", String::from_utf8(self.user_key.clone()).unwrap());
		for n in &self.ip {
			print!("{}",n);
		}
		println!("");
		/*
		for b in &self.user_key {
			print!("{}", b);
		}
		*/
		print!("\n");
	}

	pub fn read(&self) {
		print!("[");
		for n in &self.ip {
			print!("{}",n);
		}
		print!("] ");
		match std::str::from_utf8(&self.mes) {
			Ok(s) => {println!("{}",s);},
			Err(_) => {println!("");}
		}
	}
	
}

pub fn mes_from_bytes(bytes: &[u8]) -> Message {
	let mut m = Message::new(Vec::new(), Vec::new(), Vec::new());
	let mut i = 0;
	for b in bytes {	
		//272 is the size of a user key in bytes
		if i < 272 {
			m.user_key.push(*b);
		} else if i < 281 {
			m.ip.push(*b);
		} else {
			m.mes.push(*b);
		}
		i += 1;
	}
	m
}

pub fn input(tx: mpsc::Sender<Message>, user: Vec<u8>, ip: Vec<u8>) {
	println!("starting input on ip{:#?}", ip.clone());
	loop {
		//Allow sender t oenter message input
		let mut input = String::new();
		//First access the input message and read it
		io::stdin().read_line(&mut input).expect("Failed to read");
		let m: Message = Message::new(user.clone(), input.as_bytes().to_vec(), ip.clone());
		/*
		m.print();
		let bytes = m.as_bytes();
		let m2: Message = mes_from_bytes(&bytes);
		m2.print();
		*/
		//let _ = tx.send(input);
		let _ = tx.send(m);
	}
}

