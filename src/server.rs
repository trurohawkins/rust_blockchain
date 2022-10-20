use std::io;
use std::{io::{Write,Read}, time, net::{SocketAddr, TcpListener, TcpStream}, thread,
					sync::{Arc, Mutex, mpsc}};
use socket2::{Socket, Domain, Type};
use crate::{mes::{self, Message}, client::{ClientConnection}, contact::Contact, rsa};
//static mut S_VEC: Vec<TcpStream> = Vec::new();

//Handle access stream
// create a struct to hold the stream's state
//perform I/O operations
pub fn handle_sender(mut con: Arc<Mutex<ClientConnection>>, tx: mpsc::Sender<Message>, server: bool) -> io::Result<()> {
	let poo_user = rsa::User::new("Papa");
	let rec_contact: Contact = Contact::new(&poo_user);//{key: Vec::new(), name: "receipt".to_string() };
	let ip = vec!(0,0,0,0,0,0,0,0,0);
	//Handle multiple access stream
	let mut buf = [0;4096];
	loop {
		match con.try_lock() {
			Ok(mut stream) => {
				// let the receiever get a message from a sender
				match (*stream).stream.read(&mut buf) {
					Ok(bytes_read) => {
						//let bytes_read = (*stream).stream.read(&mut buf)?;
						// sender stream in a mutable variable
						if server && bytes_read == 0 {
							println!("lost connection to client");
							break;
						}
						let m: Message = mes::mes_from_bytes(&buf[..bytes_read]);
						if m.form == 6 {
							(*stream).i += 1;
							(*stream).receipt = true;
							//m.print();
							println!("recevied receipt stream incremented {}", (*stream).i);
						} else {
							let receipt: Message = Message::new(6, rec_contact.clone(), bytes_read.to_be_bytes().to_vec(), ip.clone());
							(*stream).stream.write(&receipt.as_bytes()[..]);
							let _ = tx.send(m);
						}
						

						//m.clone().print();
						/*
						let s = match std::str::from_utf8(&buf[..bytes_read]) {
							Ok(v) => v,
							Err(e) => panic!("Invalid UTF-8 sequence: {}", e)
						};
						let m: Message = Message::new(Vec::new(), s.to_string().as_bytes().to_vec());
						*/
					},
					Err(_) => {}
				}
			},
			Err(_) => {}
		}
		//stream.write(b"poo\n");//&buf[..bytes_read])?;
		// Print accceptance message
		// read, print the message sent
		//println!("from the sender:{}", String::from_utf8_lossy(&buf));
		// and you can sleep this connection wit hthe connected sender
		//thread::sleep(time::Duration::from_secs(1));
	}
	//success value
	Ok(())
}

pub fn server(receiver_listener: TcpListener, tx: mpsc::Sender<Message>, s_tx: mpsc::Sender<Arc<Mutex<ClientConnection>>>) -> io::Result<()> {
	//Enable port 7878 binding
	//let receiver_listener = TcpListener::bind("127.0.0.1:7878").expect("Failed and bind with the sender");
	//match TcpListener::bind("127.0.0.1:7878") {
	/*
	let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
	//socket.set_only_v6(false)?;
	let address: SocketAddr = "[::1]:7878".parse().unwrap();
	socket.set_reuse_address(true);
	socket.bind(&address.into())?;
	socket.bind(&address.into())?;
	socket.listen(128)?;
	let receiver_listener: TcpListener = socket.into();
	*/
	//match TcpListener::bind("0.0.0.0:7878") {
		//Ok(receiver_listener) => {
			//receiver_listener.set_nonblocking(true).expect("Cannot set non-blocking");
			// Getting a handle of the underlying thread
			let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
			println!("Server started, waiting for connections...");
			// listen to incoming connections messages and bind them to a server socket address
			for stream in receiver_listener.incoming() {
				match stream {
					Ok(stream) => {
								stream.set_read_timeout(Some(time::Duration::new(0,5000)));
								stream.set_nonblocking(true);
								let cli_con = Arc::new(Mutex::new(ClientConnection::new(stream)));
								let c_c = cli_con.clone();
						/*match stream.try_clone() {
							Ok(stream) => {*/
								let tx = tx.clone();
								let handle = thread::spawn(move || {
									handle_sender(c_c, tx, true).unwrap_or_else(|error| eprintln!("{:?}", error))
								});
								thread_vec.push(handle);
							/*},
							Err(e) => {
								println!("couldnt clone stream {}", e);
							}
						}*/
						//let cli_con = Arc::new(Mutex::new(ClientConnection::new(stream)));
						//println!("New connection!\n{:#?}", stream);
						let _ = s_tx.send(cli_con);
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
		/*
		},
		Err(e) => {
			println!("couldnt start server -- {}", e);
		}
	}
	*/
	
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

