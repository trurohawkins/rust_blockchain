use std::io;
use std::{io::{Read}, time, net::{TcpListener, TcpStream}, thread,
					sync::mpsc};
use crate::mes::{self, Message};
//static mut S_VEC: Vec<TcpStream> = Vec::new();

//Handle access stream
// create a struct to hold the stream's state
//perform I/O operations
fn handle_sender(mut stream: TcpStream, tx: mpsc::Sender<Message>) -> io::Result<()> {
	//Handle multiple access stream
	let mut buf = [0;4096];
	for _ in 0..1000{
		//tx.send("poo\n".to_string());
		// let the receiever get a message from a sender
		let bytes_read = stream.read(&mut buf)?;
		// sender stream in a mutable variable
		if bytes_read == 0 {
			println!("lost connection to client");
			return Ok(());
		}
		let m: Message = mes::mes_from_bytes(&buf[..bytes_read]);
		//m.clone().print();
		/*
		let s = match std::str::from_utf8(&buf[..bytes_read]) {
			Ok(v) => v,
			Err(e) => panic!("Invalid UTF-8 sequence: {}", e)
		};
		let m: Message = Message::new(Vec::new(), s.to_string().as_bytes().to_vec());
		*/
		let _ = tx.send(m);
		//stream.write(b"poo\n");//&buf[..bytes_read])?;
		// Print accceptance message
		// read, print the message sent
		//println!("from the sender:{}", String::from_utf8_lossy(&buf));
		// and you can sleep this connection wit hthe connected sender
		thread::sleep(time::Duration::from_secs(1));
	}
	//success value
	Ok(())
}

pub fn server(tx: mpsc::Sender<Message>, s_tx: mpsc::Sender<TcpStream>) -> io::Result<()> {
	//Enable port 7878 binding
	//let receiver_listener = TcpListener::bind("127.0.0.1:7878").expect("Failed and bind with the sender");
	match TcpListener::bind("127.0.0.1:7878") {
		Ok(receiver_listener) => {
			//receiver_listener.set_nonblocking(true).expect("Cannot set non-blocking");
			// Getting a handle of the underlying thread
			let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
			println!("Server started, waiting for connections...");
			// listen to incoming connections messages and bind them to a server socket address
			for stream in receiver_listener.incoming() {
				match stream {
					Ok(stream) => {
						match stream.try_clone() {
							Ok(stream) => {
								let tx = tx.clone();
								let handle = thread::spawn(move || {
									handle_sender(stream, tx).unwrap_or_else(|error| eprintln!("{:?}", error))
								});
								thread_vec.push(handle);
							},
							Err(e) => {
								println!("couldnt clone stream {}", e);
							}
						}
						println!("New connection!\n{:#?}", stream);
						let _ = s_tx.send(stream);
					},
					Err(e) => {
						println!("error with connection {}", e);
						return Ok(());
					}
				}
			}

			for handle in thread_vec {
				// return each single value Output contained in the heap
				let p = handle.join().unwrap();
				println!("handle {:?}", p);
			}
		},
		Err(e) => {
			println!("couldnt start server -- {}", e);
		}
	}
// success value
	Ok(())
}

unsafe fn _any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

/*
				unsafe {
					//let addr = stream.addr;
					let slice = any_as_u8_slice(&stream);
					let mut buf: Vec<u8> = Vec::new();
					for i in slice {
						println!("server:{}",*i);
						buf.push(*i);
					}
					println!("server {:#?}", stream);
					tx.send(buf);
				}
		match stream {
			Ok(mut st) => {
				//let mut s = st.expect("failed");
				//stream_vec.push(stream.try_clone().unwrap());
				// let the receiver connect with the sender
				/*
				let handle = thread::spawn(move || {
					//receiver failed to read fro mthe stream
					handle_sender(st).unwrap_or_else(|error| eprintln!("{:?}", error))
				});
				*/
				// push the ,essage in the order they are sent
				//thread_vec.push(handle);
			},
			Err(ref e) if e.kind() ==io::ErrorKind::WouldBlock => {
				//wait_for_fd();
				for mut s in &stream_vec {
					//s.write(b"poo\n");
				}
				continue;
			},
			Err(e) => panic!("encountered IO error: {e}"),
		}
		*/

