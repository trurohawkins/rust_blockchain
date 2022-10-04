mod server;
mod client;
mod rsa;
mod block;
mod mes;

use std::{io::{Write}, env, thread, sync::mpsc, net::{TcpStream}};

fn main() {//  -> io::Result<()> {
	let passphrase = "poop";
	let user1 = rsa::User::new(passphrase);
	let user2 = rsa::User::new(passphrase);
	let trans = rsa::Transaction::new(user1.public.clone(), user2.public.clone(), 0.0);

	let mut blockchain = block::Chain::new();
	blockchain.genesis(trans);
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
	let (streams_tx, streams_rx) = mpsc::channel();
	let (inp_tx, inp_rx) = mpsc::channel();
	let mut stream_vec: Vec<TcpStream> = Vec::new();
	let mut client_stream: Option<TcpStream> = None;
	
	let _handle = thread::spawn(move || {
		server::server(server_tx, streams_tx)
	});
	let _ = thread::spawn(move || {
		mes::input(inp_tx, Vec::new())
	});

	if args.len() > 1 {
			let mut ip = args[1].clone();
			ip.push_str(":7878");
			match TcpStream::connect(ip) {//.expect("couldnt connect to server");
				Ok(stream) => {
					match stream.try_clone() {
						Ok(s) => {client_stream = Some(s);},
						Err(e) => {println!("cant clone client stream {}", e);}
					}
					let _handle = thread::spawn(move || {
						client::client(stream);
					});
				},
				Err(e) => {println!("cannot connect {}", e);}
			}
			//let mut stream = TcpStream::connect(ip).expect("couldnt connect to server");
	}
	loop {
		match inp_rx.try_recv() {
			Ok(inp) => {
				//println!("got input, now I gotta send {}, is the client running? {}", inp, client_running);
				if let Some(ref mut s) = client_stream {
					//println!("we got a client stream {:#?}", s);
					let _ = s.write(&inp.as_bytes()[..]);
				}
				/*
				if client_stream {
					client_stream.write(inp.as_bytes()).expect("failed to write");
				}
				*/
			},
			Err(_) => {}
		}
		match streams_rx.try_recv() {
			Ok(s) => {
				stream_vec.push(s);
			},
			//mpsc::TryRecvError::Empty => {},
			Err(_) => {
				//println!("error retrieiving stream {}", e);
			}
		}
		match server_rx.try_recv() {
			Ok(r) => {
				 let s = match std::str::from_utf8(&r.mes[..]) {
        		Ok(v) => v,
        		Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    			};
					println!("got {}", s);
					for mut s in &stream_vec {
						let _ = s.write(&r.as_bytes()[..]);
					}
					/*
					unsafe {
						println!("{}", server::S_VEC.len());
						for s in server::S_VEC.iter_mut() {
							s.write(r.as_bytes());
						}
					}
					*/
			},
			Err(_) => {
				//println!("broadcast receive error {}", e);
			}
		}
	}
}
