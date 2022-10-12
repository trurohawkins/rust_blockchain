use std::{io, sync::mpsc};
//use serde::{Serialize, Deserialize};
use crate::{contact::{self, Contact}};

#[derive(Clone)]
pub struct Message {
	pub form: u8,
	pub user: Contact,
	pub ip: Vec<u8>,
	pub mes: Vec<u8>,
}

impl Message {
	pub fn new(form: u8, user: Contact, mes: Vec<u8>, ip: Vec<u8>) -> Self {
		Self { form, user, ip, mes }
	}

	pub fn print(&self) {
		println!("----Message----");
		print!("from: ");
		self.user.print();
		print!("IP: ");
		for n in &self.ip {
			print!("{}",n);
		}
		println!("\nmes:");
		match std::str::from_utf8(&self.mes) {
			Ok(s) => {println!("{:#?}",s);},
			Err(_) => {
				for b in &self.mes {
					print!("{} ", b);
				}
				print!("\n");
			},
		}
		/*
		println!("from\n- ");
		println!("{:#?}", self.user.name);//String::from_utf8(self.user.clone()).unwrap());
		*/
		println!("");
		print!("\n");
	}

	pub fn read(&self) {
		print!("[{}] ", self.user.name);
		match std::str::from_utf8(&self.mes) {
			Ok(s) => {println!("{}",s);},
			Err(_) => {println!("");}
		}
	}

	pub fn as_bytes(&self) -> Vec<u8> {//&[u8] {
		let mut v: Vec<u8> = Vec::new();
		v.push(self.form);
		let len: u16 = self.mes.len() as u16;
		for b in len.to_be_bytes() {
			v.push(b);
		}
		//v.push(self.mes.len() as u8);
		println!("messages leng: {}", self.mes.len());
		for e in &self.user.as_bytes() {
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
	
}

pub fn mes_from_bytes(bytes: &[u8]) -> Message {
	let mut u: Vec<u8> = Vec::new();
	let mut ip: Vec<u8> = Vec::new();
	let mut mes: Vec<u8> = Vec::new();
	let form = bytes[0];
	let len = ((bytes[1] as u16) << 8) | bytes[2] as u16; 
	//let len: u32 = bytes[1].into();
	let mut i = 0;
	//let c_len: u32 = 272 + (bytes[0]).into();
	let mut c_len: u16 = bytes[3].into();
	c_len += 273 + 3;
	for b in bytes {	
		//273 is the size of a user key in bytes
		if i > 2 {
			if i < c_len {
				u.push(*b);
			} else if i < c_len + 9 {
				ip.push(*b);
			} else if i < c_len + 9 + len {
				mes.push(*b);
			}
		}
		i += 1;
	}
	/*let mut m = Message::new(Vec::new(), Vec::new(), Vec::new());
	m*/
	Message { form, user: contact::bytes_to_contact(u), ip, mes }
}

pub fn input(tx: mpsc::Sender<String>) {
	//let mut main_user: Option<User> = None;
	//let mut my_contact: Contact = Default::default();
	//let mut my_contact: Option<Contact> = None;
	//println!("starting input on ip{:#?}", ip.clone());
	/*
	match fs::read("user.sav") {
		Ok(v) => {
			let u = rsa::bytes_to_user(v);
			my_contact = Contact::new(&u);
			main_user = Some(u.clone());
			//println!("welcome back, {}...", u.name.clone());
		}
		Err(_) => {
			//println!("Hello and Welcome, Please enter a name t ocontinue\n");
		}	
	}
*/
	let mut first_inp = true;
	loop {
		if !first_inp {
			//Allow sender t oenter message input
			let mut input = String::new();
			//First access the input message and read it
			io::stdin().read_line(&mut input).expect("Failed to read");
			match tx.send(input) {
				Ok(_) => {},
				Err(e) => {println!("input broadcast to main thread failed {}", e);}
			}
		} else {
			match tx.send("C".to_string()) {
				Ok(_) => {},
				Err(e) => {println!("input broadcast to main thread failed {}", e);}
			}
			first_inp = false;
		}
		//}
		/*
		match main_user {
			Some(ref user) => {
				//let data = parse_command(input, book, user.clone());
				let m: Message = Message::new(my_contact.clone(), input.as_bytes().to_vec(), ip.clone());
				let _ = tx.send(m);
			},
			None => {
				let s = &input[..input.len()-1];
				println!("Creating new User, {}", s);
				main_user = Some(User::new(&s));
				match main_user {
					Some(ref u) => {
						u.print();
						//u.save();
						my_contact = Contact::new(&u);
					},
					None => todo!(),
				}
			}
		}
		*/
	}
}
