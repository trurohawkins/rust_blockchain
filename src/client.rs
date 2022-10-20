use std::{net::TcpStream, io::{self, prelude::*,BufReader}, sync::mpsc,
	sync::{Arc, Mutex}};
use crate::{mes::{self, Message}, contact::Contact, rsa};

pub struct ClientConnection {
	pub stream: TcpStream,
	pub user: Vec<u8>,
	pub i: u8,
	pub receipt: bool
}

impl ClientConnection {
	pub fn new(stream: TcpStream) -> Self {
		let user: Vec<u8> = Vec::new();
		Self {stream, user, i: 0, receipt: true}
	}
/*
	pub fn add_user(&mut self, u: Vec<u8>) {
		for b in u {
			self.user.push(b);
		}
	}

	pub fn check_user(&self) -> bool {
		if self.user.len() == 0 {
			false
		} else {
			true
		}
	}
*/
}

pub fn client(mut stream: TcpStream, c_tx: mpsc::Sender<Message>) -> io::Result<( )> {
	//connect
	//Struct used to start requests to the server
	// Check TcpStream Connection to the server
	//ip.push_str(":7878");
	//let mut stream = TcpStream::connect(ip).expect("couldnt connect to server");
	
	//stream.set_nonblocking(true).expect("set_nonblocking call failed");
	//let mut buf = vec![];A
	let poo_user = rsa::User::new("Papa");
	let rec_contact: Contact = Contact::new(&poo_user);//{key: Vec::new(), name: "receipt".to_string() };
	let ip = vec!(0,0,0,0,0,0,0,0,0);
	loop {
	//for _ in 0..1000 {
		//Write the message so that the receiver can access it
		//stream.write(input.as_bytes()).expect("failed to write");
	
		// add buffering so that the receiver can read messages from the stream
		let mut reader = BufReader::new(&stream);
		// Check if this input message vales are u8
		//let mut buffer: Vec<u8> = Vec::new();
		let mut buf = [0;4096];
		// Read the input information
		match reader.read(&mut buf) {//_to_end(&mut buffer);
			Ok(bytes_read) => {
				if bytes_read == 0 {
					println!("ohnoohno zerooo bytes, server is probably gone, I will disconnect now");
					break;
				}
				println!("sending receipt of {} in length", bytes_read);
				let receipt: Message = Message::new(6, rec_contact.clone(), bytes_read.to_be_bytes().to_vec(), ip.clone());
				let _ = stream.write(&receipt.as_bytes()[..]);

				let m: Message = mes::mes_from_bytes(&buf);
				match c_tx.send(m.clone()) {
					Ok(_) => {},
					Err(e) => {panic!("error sending broadcast from client thread {}", e);}
				}
				/*
				let s = match std::str::from_utf8(&m.mes[..]) {
					Ok(v) => v,
					Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
				};
				*/
				//let c = m.clone().user;
				//book.add_contact(c);
				//book.print();
				//m.read();
			},
			Err(e) => {
				println!("client error, reading from stream {}", e);
			}
		}
		/*
		match stream.read_to_end(&mut buf) {
			Ok(_) => {
				println!("read from server:{}", str::from_utf8(&buf).unwrap());
				break;
			},
			Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
			},
			Err(e) => panic!("encountered IO error: {e}"),
		}
		println!("read from server:{}", str::from_utf8(&buffer).unwrap());
		println!("");
		*/
	//}
	}
	Ok(())
}
