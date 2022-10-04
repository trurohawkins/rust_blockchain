use std::{net::TcpStream, io::{prelude::*,BufReader}};

pub fn client(stream: TcpStream) { //-> io::Result<( )> {
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
		let mut buffer: Vec<u8> = Vec::new();
		// Read the input information
		let _ = reader.read_until(b'\n', &mut buffer);
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
	//Ok(())
}
