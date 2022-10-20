use chrono::Utc;
use std::fs;
use serde::{Deserialize, Serialize};
use openssl::{rsa::{Rsa, Padding}, symm::Cipher};
use bincode;

#[derive(Clone)]
pub struct User {
	private: Vec<u8>,
	pub public: Vec<u8>,
	pub name: String 
}

impl User {
	pub fn new(name: &str) -> Self {
		let rsa = Rsa::generate(1024).unwrap();

		let private: Vec<u8> = rsa.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), name.as_bytes()).unwrap();
		let public: Vec<u8> = rsa.public_key_to_pem().unwrap();
		Self { private, public, name: name.to_string() }	
	}

	pub fn print(&self) {
		println!("{}", self.name);
		println!("Private key: {}", String::from_utf8(self.private.clone()).unwrap());
		println!("Public key: {}", String::from_utf8(self.public.clone()).unwrap());
	}
	
	fn as_bytes(&self) -> Vec<u8> {
		let mut v = Vec::new();
		for b in &self.private {
			v.push(*b);
		}
		for b in &self.public {
			v.push(*b);
		}
		for b in self.name.as_bytes() {
			v.push(*b);
		}
		v
	}
	
	pub fn save(&self) {
		let b = self.as_bytes();
		match fs::write("user.sav", b) {
			Ok(_) => {},
			Err(e) => { panic!("failed to save user file {}", e); }
		}
	}
}

pub fn bytes_to_user(bytes: Vec<u8>) -> User {
	let mut private = Vec::new();
	let mut public = Vec::new();
	let mut n = Vec::new();
	let mut i = 0;
	for b in bytes {
		if i < 986 {
			private.push(b);
		} else if i < 1258 {
			public.push(b);
		} else {
			n.push(b);
		}
		i += 1;
	}
	let name = match std::str::from_utf8(&n) {
		Ok(v) => v,
		Err(e) => panic!("bad contact buff {}", e),
	};
	User {public, private, name: name.to_string()}
}


#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
	//sender: (BigUint, BigUint),
	//recipient: (BigUint, BigUint), 
	pub sender: Vec<u8>,//Rsa<Public>,
	//sender: rsa::RsaPublicKey,//Rsa<Public>,
	pub recipient: Vec<u8>,//Rsa<Public>,
	//recipient: rsa::RsaPublicKey,//Rsa<Public>,
	pub amount: f64,
	pub timestamp: i64
}

impl Transaction {
	pub fn new(sender: Vec<u8>, recipient: Vec<u8>, amount: f64) -> Self {
		let timestamp = Utc::now().timestamp();
		Self { sender, recipient, amount, timestamp }
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransHash {
	buff: Vec<Vec<u8>>,
	chunk: usize
}

impl TransHash {
	fn new(buff: Vec<Vec<u8>>, chunk: usize) -> Self {
		Self {buff, chunk}
	}
}

pub fn sign_transaction(sender: User, recipient: Vec<u8>, amount: f64) -> TransHash { //trans: Transaction, private: Vec<u8>, name: &str) -> Vec<Vec<u8>> {
		let trans = Transaction::new(sender.public, recipient, amount);
		let d: Vec<u8> = bincode::serialize(&trans).unwrap();
		let chunk = d.len() / 8;//100;
		//println!("data size {}", d.len());
		let mut cur = 0;
		let mut buff_arr = Vec::new();
	
		while cur < d.len() {
			//println!("cur: {}", cur);
			let mut data = Vec::<u8>::new();
			let mut i = 0;
			while i < chunk && cur < d.len() {
				//print!("{} ", d[cur]);
				if cur < d.len() {
					data.push(d[cur]);
				} else {
					data.push(0);
				}
				i += 1;
				cur += 1;
			}
			//println!("\n");
			//let decoded: Vec<u8> = bincode::deserialize(&encoded[..]).unwrap();
			//print_key(decoded);

			//let data = "Jerry the pump dick";
			let private_pem = String::from_utf8(sender.private.clone()).unwrap();
			let private_key = Rsa::private_key_from_pem_passphrase(private_pem.as_bytes(), sender.name.as_bytes()).unwrap();
			//println!("data size {}, and key size: {}", d.len(), private_key.size());
			let mut buf: Vec<u8> = vec![0; private_key.size() as usize];
			let _ = private_key.private_encrypt(&data/*.as_bytes()*/, &mut buf, Padding::PKCS1).unwrap();
			buff_arr.push(buf.clone());
		}
	TransHash::new(buff_arr, chunk)
}

pub fn print_transaction(trans :&Transaction) {
  print!("TRANSACTION:\n{}", String::from_utf8(trans.sender.clone()).unwrap());
	println!("IS SENDING {} TO", trans.amount);
  print!("{}", String::from_utf8(trans.recipient.clone()).unwrap());
	println!("AT {}\n", trans.timestamp);
}

fn print_key(key: Vec<u8>) {
	println!("{}", String::from_utf8(key).unwrap());
}

pub fn verify_transaction(sender: Vec<u8>, th: TransHash) -> Option<Transaction> {
	let mut decoded = Vec::new();
	let mut cur = 0;
	while cur < th.buff.len() {
		let data = &th.buff[cur];
		let public_pem = String::from_utf8(sender.clone()).unwrap();
		let public_key = Rsa::public_key_from_pem(public_pem.as_bytes()).unwrap();
		let mut buf: Vec<u8> = vec![0; public_key.size() as usize];

		//let _ = public_key.public_decrypt(&data, &mut buf, Padding::PKCS1).unwrap();
		match public_key.public_decrypt(&data, &mut buf, Padding::PKCS1) {
			Ok(_) => {
				let mut i = 0;
				while i < th.chunk {//&& i < buf.len() {
					decoded.push(buf[i]);
					i += 1;
				}
				cur += 1;
			}, 
			Err(e) => {
				println!("cant decrypt public key {}", e);
				return None;
			}
		} 
	}
	//let t: Transaction = bincode::deserialize(&decoded[..]).unwrap();
	Some(bincode::deserialize(&decoded[..]).unwrap())
}
