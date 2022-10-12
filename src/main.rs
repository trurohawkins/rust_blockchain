mod server;
mod client;
mod rsa;
mod block;
mod mes;
mod contact;

use std::{io::{Write}, time, env, thread, sync::mpsc, net::{TcpStream}};

fn main() {//  -> io::Result<()> {
	let mut c_book = contact::ContactBook::new();
	let mut blockchain: block::Chain<Vec<rsa::Transaction>> = block::Chain::new();
	blockchain.genesis(Vec::new());
	/*
	let passphrase = "!Poop";
	let user1 = rsa::User::new(passphrase);
	user1.save();
	user1.print();
	thread::sleep(time::Duration::from_secs(1));
	let c = contact::Contact::new(&user1);
	c_book.add_contact(c.clone());
	*/
	/*
	let c2 = rsa::bytes_to_Contact(c.as_bytes());
	println!("");
	c.print();
	println!("\n~~~~~~\n");
	c2.print();
	println!("");
	let user2 = rsa::User::new(passphrase);
	let trans = rsa::Transaction::new(user1.public.clone(), user2.public.clone(), 0.0);
	*/
	/*
	let th = rsa::sign_transaction(user1.clone(), user2.public.clone(), 0.0);
	let t = rsa::verify_transaction(user1.public, th.clone());
	let latest_block = blockchain.blocks.last().expect("at leastt one block"); 
	let b = block::Block::new(latest_block.id + 1, latest_block.hash.clone(), t.clone());
	blockchain.try_add_block(b);
	*/
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
	let mut client_stream: Option<TcpStream> = None;
	let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
	
	let hand = thread::spawn(move || {
		server::server(server_tx, streams_tx);
	});
	threads.push(hand);
	thread::sleep(time::Duration::from_secs(1));
	let mut connections: Vec<client::ClientConnection> = Vec::new();
	let mut my_ip: Vec<u8> = Vec::new();
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
							client_stream = Some(s);
						},
						Err(e) => {println!("cant clone client stream {}", e);}
					}

					let handle = thread::spawn(move || {
						client::client(stream, client_tx);
					});
					threads.push(handle);
				},
				Err(e) => {println!("cannot connect {}", e);}
			}
			//let mut stream = TcpStream::connect(ip).expect("couldnt connect to server");
	}

	let handle = thread::spawn(move || {
			mes::input(inp_tx)
	});
	threads.push(handle);

	let mut closing_time = false;
	loop {
		match inp_rx.try_recv() {
			Ok(inp) => {
				let letter: u8 = inp.chars().next().unwrap() as u8;
				if letter == 27 {
				//if inp.len() == 0 {// && inp.ip.len() == 0 {
					println!("received empty message");
					closing_time = true;
				} else if let Some(ref mut s) = client_stream {
					//println!("we got a client stream {:#?}", s);
					let m = c_book.parse_command(inp, my_ip.clone(), &mut blockchain);
					match m {
						Some(message) => {
							let _ = s.write(&message.as_bytes()[..]);
						},
						None => {},
					}
				}
			},
			Err(_) => {}
		}
		match streams_rx.try_recv() {
			Ok(s) => {
				//stream_vec.push(s);
				connections.push(client::ClientConnection::new(s));
			},
			//mpsc::TryRecvError::Empty => {},
			Err(_) => {
				//println!("error retrieiving stream {}", e);
			}
		}
		match server_rx.try_recv() {
			Ok(r) => {
				let from = r.ip.clone();//user_key.clone();
				//for c in &mut connections {
				let mut i =0;
				'con: while i < connections.len() {
					let c = &mut connections[i];
					match c.stream.peer_addr() {
						Ok(a) => {
							let ip: String = a.ip().to_string();
							let port: String = a.port().to_string();
							let cur = ip_to_vec(ip, port);
							if cur != from {
								let _ = c.stream.write(&r.clone().as_bytes()[..]);
							} else {
							}
						},
						Err(e) => {
							println!("we got an error{}", e);
							// remove from list
							connections.remove(i);
							continue 'con;
						}
					}
					i += 1;
				}
				/*
				let s = match std::str::from_utf8(&r.mes[..]) {
        	Ok(v) => v,
        	Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    		};
				println!("got {}", s);
				*/
			},
			Err(_) => {
				//println!("broadcast receive error {}", e);
			}
		}
		match client_rx.try_recv() {
			Ok(m) => {
				let mut reply = c_book.parse_message(m, my_ip.clone(), &mut blockchain);
				if let Some(ref mut r) = reply {
					if let Some(ref mut s) = client_stream {
						let _ = s.write(&r.as_bytes()[..]);
					}
				}

				/*
				m.parse();
				c_book.add_contact(m.user);
				c_book.print();
				*/
			},
			Err(_) => {
				//println!("receive from client thread error");
			}
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
				Ok(p) => {
					println!("handle {:?}, \nnew size: {}", p, threads.len());
				},
				Err(e) => {
					println!("error from thread closure {:#?}", e);
				}
			}
		}
		if closing_time || threads.len() == 1 {
			println!("breaking loop");
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

