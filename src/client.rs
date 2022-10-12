use std::{net::TcpStream, io::{self, prelude::*,BufReader}, sync::mpsc};
use crate::{mes::{self, Message}};

pub struct ClientConnection {
	pub stream: TcpStream,
	pub user: Vec<u8>
}

impl ClientConnection {
	pub fn new(stream: TcpStream) -> Self {
		let user: Vec<u8> = Vec::new();
		Self {stream, user}
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

pub fn client(stream: TcpStream, c_tx: mpsc::Sender<Message>) -> io::Result<( )> {
	//connect
	//Struct used to start requests to the server
	// Check TcpStream Connection to the server
	//ip.push_str(":7878");
	//let mut stream = TcpStream::connect(ip).expect("couldnt connect to server");
	
	//stream.set_nonblocking(true).expect("set_nonblocking call failed");
	//let mut buf = vec![];
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

				let m: Message = mes::mes_from_bytes(&buf);
				match c_tx.send(m) {
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
