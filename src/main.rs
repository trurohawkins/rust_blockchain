mod server;
mod client;
mod rsa;
mod block;
mod mes;
mod contact;

use std::{io::{Write}, time, env, thread, sync::{Arc, Mutex, mpsc}, net::{TcpStream, TcpListener}};
use crate::{mes::{Message}, client::ClientConnection};

fn main() {//  -> io::Result<()> {
	let mut c_book = contact::ContactBook::new();
	let mut blockchain: block::Chain = block::Chain::new();
	if blockchain.is_chain_valid(&blockchain.blocks) {
		println!("valid chain");
	} else {
		println!("invalid chain");
	}
	let args: Vec<String> = env::args().collect();
	let (server_tx, server_rx) = mpsc::channel();
	let (client_tx, client_rx) = mpsc::channel();
	let (streams_tx, streams_rx) = mpsc::channel();
	let (inp_tx, inp_rx) = mpsc::channel();
	let mut client_stream: Option<Arc<Mutex<ClientConnection>>> = None;// = Arc::new(Mutex::new(Default::default()));
	let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
	let mut server_started = false;	
	match TcpListener::bind("0.0.0.0:7878") { 
		Ok(listener) => {
			server_started = true;
			let hand = thread::spawn(move || {
				match server::server(listener, server_tx, streams_tx) {
					Ok(_) => {},
					Err(_) => {}
				}
			});
			threads.push(hand);
		},
		Err(e) => {
			println!("unable to start server {}", e);
		}
	}

	//thread::sleep(time::Duration::from_secs(1));
	let mut connections: Arc<Mutex<Vec<Arc<Mutex<ClientConnection>>>>> = Arc::new(Mutex::new(Vec::new()));
	let mut my_ip: Vec<u8> = Vec::new();
	let mut client_started = false;
	if args.len() > 1 {
			let mut ip = args[1].clone();
			ip.push_str(":7878");
			match TcpStream::connect(ip) {//.expect("couldnt connect to server");
				Ok(stream) => {
					match stream.try_clone() {
						Ok(s) => {
							match s.local_addr() {
								Ok(a) => {
									let ip: String = a.ip().to_string();
									let port: String = a.port().to_string();
									my_ip = ip_to_vec(ip, port);
								},
								Err(_) => {}
							}
							//my_ip = ip_to_vec(s.clone());	
							println!("I am connected {:#?}", s);
							//client_stream = Some(s);
						},
						Err(e) => {println!("cant clone client stream {}", e);}
					}
					stream.set_read_timeout(Some(time::Duration::new(0,5000)));
					stream.set_nonblocking(true);
					let connection = Arc::new(Mutex::new(ClientConnection::new(stream)));
					let cs = connection.clone();
					client_stream = Some(connection);
					client_started = true;
					let handle = thread::spawn(move || {
						//match client::client(stream, client_tx) {
						match server::handle_sender(cs, client_tx, false) {
							Ok(_) => {},
							Err(_) => {}
						}
					});
					threads.push(handle);
				},
				Err(e) => {println!("cannot connect {}", e);}
			}
			//let mut stream = TcpStream::connect(ip).expect("couldnt connect to server");
	} else {
		let ip = "0.0.0.0".to_string();
		let port = "00000".to_string();
		my_ip = ip_to_vec(ip, port);
	}

	let handle = thread::spawn(move || {
			mes::input(inp_tx)
	});
	threads.push(handle);

	let mut to_clients: Vec<Message> = Vec::new();
	let mut to_server: Vec<Message> = Vec::new();
	let mut closing_time = false;
	let mut all_received = 0;

	loop {
		match streams_rx.try_recv() {
			Ok(s) => {
				println!("got something from stream");
				let mut con = connections.lock().unwrap();
				//stream_vec.push(s);
				//connections.push(s);
				(*con).push(s);
				let chain_length: u64 = blockchain.blocks.len().try_into().unwrap();
				let m: Message = Message::new(4, c_book.my_contact.clone(), chain_length.to_be_bytes().to_vec(), my_ip.clone());
				let new_len = (*con).len();
				let mut c = (*con)[new_len-1].lock().unwrap();
				//let _ = c.stream.write(&m.clone().as_bytes()[..]);
				println!("finished wit hthat stream");
			},
			//mpsc::TryRecvError::Empty => {},
			Err(_) => {
				//println!("error retrieiving stream {}", e);
			}
		}
		// input creating
		match inp_rx.try_recv() {
			Ok(inp) => {
				let letter: u8 = inp.chars().next().unwrap() as u8;
				if letter == 27 {
					closing_time = true;
				} else if letter == 49 {
					for m in &to_clients {
						println!("m: type {} from {:#?}", (*m).form, (*m).ip);
					}
					let con = connections.lock().unwrap();
					for c in &*con {
						let cc = c.lock().unwrap();
						println!("{:#?} currently at {}", cc.user, cc.i);
					}

				} else {
					let m = c_book.parse_command(inp, my_ip.clone(), &mut blockchain);
					match m {
						Some(message) => {
							if server_started {
								println!("pushing command to be sent");
								to_clients.push(message.clone());
							}
							//if let Some(_) = client_stream {
							if client_started {
								to_server.push(message.clone());
							}
						},
						None => {},
					}
				}
			},
			Err(_) => {}
		}
		//local server received from client
		match server_rx.try_recv() {
			Ok(m) => {
				println!("got something from a client");
				to_clients.push(m.clone());
				let mut reply = c_book.parse_message(m, my_ip.clone(), &mut blockchain);
				if let Some(ref mut r) = reply {
					if client_started {
					//if let Some(_) = client_stream {
						to_server.push(r.clone());
					}
					if server_started {
						to_clients.push(r.clone());
					}
				}
			},
			Err(_) => {
				//println!("broadcast receive error {}", e);
			}
		}
		//local client received from remote server
		match client_rx.try_recv() {
			Ok(m) => {
				//to_server.push(m.clone());
				let mut reply = c_book.parse_message(m, my_ip.clone(), &mut blockchain);
				if let Some(ref mut r) = reply {
					if server_started {
						to_clients.push(r.clone());
					}
				}
			},
			Err(_) => {
				//println!("receive from client thread error");
			}
		}
		if to_server.len() > 0 {
			//let mes = to_server.remove(0);
			if let Some(ref mut client_stream) = client_stream {
				match client_stream.try_lock() {
					Ok(mut con) => {
						if (*con).receipt {
							let cur = (*con).i as usize;
							if usize::from((*con).i) < to_server.len() {
								let mes = to_server[cur].clone();
								let _ = (*con).stream.write(&mes.as_bytes()[..]);
								(*con).receipt = false;
							}
						} else if usize::from((*con).i) >= to_server.len() {
							to_server.clear();
							(*con).i = 0;
						}
					},
					Err(_) => {}
				}
			}
		}
		if to_clients.len() > 0 {
			match connections.try_lock() {
				Ok(mut conn) => {
					let mut i = 0;
					'con: while i < conn.len() {
						let c = Arc::clone(&conn[i]);
						match c.try_lock() {
							Ok(mut con) => {
								let cur = (*con).i as usize;
								if (*con).receipt {
								if usize::from((*con).i) < to_clients.len()  {
									println!("client #{} at {} of {}",i, (*con).i, to_clients.len());
									let mes = to_clients[cur].clone();//to_clients.remove(0);
									let from = mes.ip.clone();
									//to_clients.remove(0);
									match (*con).stream.peer_addr() {
										Ok(a) => {
											let ip: String = a.ip().to_string();
											let port: String = a.port().to_string();
											let cur = ip_to_vec(ip, port);
											if cur != from {
												println!("sent!");
												let _ = (*con).stream.write(&mes.as_bytes()[..]);
												(*con).receipt = false;
											} else {
												(*con).i += 1;
												println!("incremented con: {}", (*con).i);
											}
										},
										Err(e) => {
											println!("we got an error{}", e);
											// remove from list
											conn.remove(i);
											continue 'con;
										}
									}
								}
									println!("{} >= {}?", usize::from((*con).i), to_clients.len());
									if usize::from((*con).i) >= to_clients.len() {
										all_received += 1;
									}

								}
								//println!("singular unlocked");
							},
							Err(_) => {}//println!("couldnt unlock {}", e);}
						}
						i += 1;
					}
					if (*conn).len() > 0 && all_received >= (*conn).len() {
						let mut len = (*to_clients).len();
								loop {
									for co in &*conn {
										let mut c = co.lock().unwrap();
										(*c).i = 0;
									}
									println!("remvoing from to_clients");
									while len > 0 {
										to_clients.remove(0);
										len -= 1;
									}
									all_received = 0;
									break;
								}
							}
				},
				Err(_) => {}
			}
			/*
			let mut reply = c_book.parse_message(mes, my_ip.clone(), &mut blockchain);
			if let Some(ref mut r) = reply {
				println!("sending our reply");
				to_clients.push(r.clone());
				to_server.push(r.clone());
			}
			*/
		}
		let mut to_remove = Vec::new();
		//println!("thread size: {}", threads.len());
		for i in 0..threads.len() {
			if threads[i].is_finished() {
				to_remove.push(i);
			}
		}
		for i in &to_remove {
			let t = threads.remove(*i);
			match t.join() {//.unwrap();
				Ok(_) => {
					//println!("handle {:?}, \nnew size: {}", p, threads.len());
				},
				Err(e) => {
					println!("error from thread closure {:#?}", e);
				}
			}
		}
		if closing_time || threads.len() == 1 {
			//println!("breaking loop");
			break;
		}
		/*
		for handle in thread_vec {
			// return each single value Output contained in the heap
			if handle.is_finished() {
			}
		}
		*/
	}
	blockchain.save_chain();
}

fn ip_to_vec(ip: String, port: String) -> Vec<u8> {
	let mut nums: Vec<u8> = Vec::new();
	/*
	match stream.peer_addr() {
		Ok(a) => {
			let s: String = a.ip().to_string();
		*/
			let mut strings: Vec<String> = Vec::new();
			let mut cur: Vec<char> = Vec::new();
			for c in ip.chars() {
				if c.is_digit(10) {
					cur.push(c);
				} else {
					strings.push(cur.iter().cloned().collect::<String>());
					cur = Vec::new();
				}
			}
			strings.push(cur.iter().cloned().collect::<String>());
			cur = Vec::new();
			for c in port.chars() {
				if c.is_digit(10) {
					cur.push(c);
					strings.push(cur.iter().cloned().collect::<String>());
					cur = Vec::new();
				}
			}
			strings.push(cur.iter().cloned().collect::<String>());

			for n in strings {
				match n.parse::<u8>() {
					Ok(int) => {
						nums.push(int);
					},
					Err(_) => {}
				}
			}
	/*	},
		Err(e) => {println!("error getting ip {}", e);}
	//} */
	nums
}

