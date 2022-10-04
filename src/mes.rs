use std::{io, sync::mpsc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
	pub user_key: Vec<u8>,
	pub mes: Vec<u8> 
}

impl Message {
	pub fn new(user_key: Vec<u8>, mes: Vec<u8>) -> Self {
		Self { user_key, mes }
	}
	pub fn as_bytes(&self) -> Vec<u8> {//&[u8] {
		let mut v: Vec<u8> = Vec::new();
		for e in &self.user_key {
			v.push(*e);
		}
		for e in &self.mes {
			v.push(*e);
		}
		v
		//let bytes: [u8] = v.as_slice();
		//let bytes: &[u8] = &v[..];
		//bytes
	}
}

pub fn input(tx: mpsc::Sender<Message>, user: Vec<u8>) {
	loop {
		//Allow sender t oenter message input
		let mut input = String::new();
		//First access the input message and read it
		io::stdin().read_line(&mut input).expect("Failed to read");
		let m: Message = Message::new(user.clone(), input.as_bytes().to_vec());
		//let _ = tx.send(input);
		let _ = tx.send(m);
	}
}

