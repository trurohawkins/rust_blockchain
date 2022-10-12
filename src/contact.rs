use std::{io, fs};
use  crate::{rsa::{self, User}, mes::{Message},block};

pub struct ContactBook {
	main_user: Option<User>,
	my_contact: Contact,
	book: [Vec<Contact>; 27],
	transaction_pool: Vec<rsa::Transaction>
}

impl ContactBook {
	pub fn new() -> Self {
		let mut main_user: Option<User> = None;
		let mut my_contact: Contact = Default::default();
		match fs::read("user.sav") {
			Ok(v) => {
				let u = rsa::bytes_to_user(v);
				my_contact = Contact::new(&u);
				main_user = Some(u.clone());
				println!("welcome back, {}...", u.name.clone());
			}
			Err(_) => {
				println!("Hello and Welcome, Please enter a name to continue...");
				let mut input = String::new();
				//First access the input message and read it
				io::stdin().read_line(&mut input).expect("Failed to read");
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
		let book: [Vec<Contact>;27] = Default::default();//[0;27];
		let transaction_pool = Vec::new();
		/*
		for _ in 0..27 {
			book.push(Vec::new());
		}
		*/
		Self { main_user, my_contact, book, transaction_pool }
	}

	pub fn add_contact(&mut self, c: Contact) -> bool {
		let num = get_index(c.clone().name);
		for con in &self.book[num] {
			if con.key == c.key {
				return false;
			}
		}
		println!("added {}", c.name);
		self.book[num].push(c.clone());
		true
	}

	pub fn find_contact(&self, name: String) -> Option<Contact> {
		//println!("searching for {:#?}", name.clone());
		let index = get_index(name.clone());
		for con in &self.book[index] {
			//println!("is it {:#?}?", con.name.clone());
			if con.name == name {
				return Some(con.clone());
			}
		}
		None
	}

	pub fn parse_command(&mut self, input: String, ip: Vec<u8>, chain: &mut block::Chain<Vec<rsa::Transaction>>) -> Option<Message> {
		match self.main_user {
			Some(ref user) => {
				let v: Vec<&str> = input.splitn(3, " ").collect();
				let letter: char = v[0].chars().next().unwrap() as char;

				if letter == 't' || letter == 'T' {
					if v.len() < 3 {
						return None;
					}
					match v[1].parse::<f64>() {
						Ok(amount) => {
							println!("sending {} to {}", amount, v[2]);
							let recipient = self.find_contact(v[2].trim_matches('\n').to_string());
							match recipient {
								Some(r) => {
									//let trans = rsa::Transaction::new(user.public.clone(), r.key, amount);
									let hash = rsa::sign_transaction(user.clone(), r.key, amount);
									let d: Vec<u8> = bincode::serialize(&hash).unwrap();
									/*
									rsa::print_transaction(trans);
									*/
									let th: rsa::TransHash = bincode::deserialize(&d).unwrap();
									match rsa::verify_transaction(user.public.clone(), th) {
										Some(trans) => {
											self.transaction_pool.push(trans);
										},
										None => {println!("unable to add transaction D:");}
									}
									let m: Message = Message::new(1, self.my_contact.clone(), d, ip.clone());
									//m.parse();
									return Some(m);
								},
								None => {
									println!("couldn't find {} in contacts", v[2].to_string());
									return None
								}
							}
						},
						Err(e) => {
							println!("couldn't get float from command {}: {}", input, e);
							return None;
						}
					}
				} else if letter == 'c' || letter == 'C' {
					let d: Vec<u8> = self.my_contact.as_bytes();
					let m: Message = Message::new(2, self.my_contact.clone(), d, ip.clone());
					return Some(m);
				} else if letter == 'm' || letter == 'M' {
					let mut i = 0;
					let mut tran_vec: Vec<rsa::Transaction> = Vec::new();
					while i < 3 {
						if self.transaction_pool.len() > 0 {
							let t = self.transaction_pool.remove(0);
							tran_vec.push(t);
						} else {
							break;
						}
						/*
						match t {
							Some(trans) => {
								tran_vec.push(trans);
							},
							None => {
								break;
							}
						}
							*/
						i += 1;
					}
					println!("mining: got {} transactions", i);
					if i > 0 {
						let latest_block = chain.blocks.last().expect("at least one block");
						let b = block::Block::new(latest_block.id + 1, latest_block.hash.clone(), tran_vec);
						let d = bincode::serialize(&b).unwrap();
						chain.try_add_block(b.clone());
						block::print_transaction_block(&b);
						let m: Message = Message::new(3, self.my_contact.clone(), d,ip.clone());
						return Some(m);
					} else {
						return None;
					}
				} else if letter == 'l' || letter == 'L' {
					let chain_length: u64 = chain.blocks.len().try_into().unwrap();
					let m: Message = Message::new(4, self.my_contact.clone(), chain_length.to_be_bytes().to_vec(), ip.clone());
					return Some(m);
				}
				//let data = parse_command(input, book, user.clone());
				let m: Message = Message::new(0, self.my_contact.clone(), input.as_bytes().to_vec(), ip.clone());
				return Some(m);
			},
			None => {
				return None;
			}
		}
	}
	pub fn parse_message(&mut self, m: Message, ip: Vec<u8>, chain: &mut block::Chain<Vec<rsa::Transaction>>) -> Option<Message> {
		println!("Parsing {}", m.form);
		if m.form == 0 {
			m.read();
			return None;
		} else if m.form == 1 {
			m.print();
			match bincode::deserialize(&m.mes) {
				Ok(th) => {
					//let trans = rsa::verify_transaction(m.user.key.clone(), th);
					match rsa::verify_transaction(m.user.key.clone(), th) {
						Some(trans) => {
							rsa::print_transaction(&trans);
							self.transaction_pool.push(trans);
						},
						None => { println!("unable to add transaction"); }
					}
					return None;
				},
				Err(e) => {
					println!("couldn't deserialize message {}", e);
					m.print();
					return None;
				}
			}
			//let th: rsa::TransHash = bincode::deserialize(&mes).unwrap();
		} else if m.form == 2 {
			//contact messgae
			let c: Contact = bytes_to_contact(m.mes.clone());
			if self.add_contact(c) {
				let d: Vec<u8> = self.my_contact.as_bytes();
				let m: Message = Message::new(2, self.my_contact.clone(), d, ip.clone());
				return Some(m);
			} else {
				return None;
			}
		} else if m.form == 3 {
			match bincode::deserialize(&m.mes) {
				Ok(block) => {
					if chain.try_add_block(block) {
						self.prune_transactions(chain);
					}
					block::chain_print_transactions(chain);
				},
				Err(e) => {
					println!("couldn't unpack block {}", e);
				}
			}
		} else if m.form == 4 {
			let mut shift = 56;
			let mut other_len: u64 = (m.mes[0] as u64) << shift;
			let mut i = 1;
			while shift > 0 {
				shift -= 8;
				other_len = other_len | (m.mes[i] as u64) << shift;
				i += 1;
			}
			let our_len: u64 = chain.blocks.len().try_into().unwrap();
			println!("got length: {},  ours: {}",other_len, our_len );
			if other_len > our_len {
				//send chain length
				let chain_length: u64 = chain.blocks.len().try_into().unwrap();
				let m: Message = Message::new(4, self.my_contact.clone(), chain_length.to_be_bytes().to_vec(), ip.clone());
				return Some(m);
			} else if other_len < our_len {
				//send our chain
				let d = bincode::serialize(&chain).unwrap();
				let m: Message = Message::new(5, self.my_contact.clone(), d, ip.clone());
				return Some(m);
			}
		} else if m.form == 5 {
			match bincode::deserialize::<block::Chain<Vec<rsa::Transaction>>>(&m.mes) {
				Ok(other_chain) => {
					chain.blocks = chain.choose_chain(chain.blocks.clone(), other_chain.blocks);
					block::chain_print_transactions(&chain);
				},
				Err(e) => {println!("couldn't unpack remote blockchain {}", e);}
			}
		}
		None
	}

	pub fn prune_transactions(&mut self, chain: &mut block::Chain<Vec<rsa::Transaction>>) {
		let size = self.transaction_pool.len();
		let mut i = 0;// size - 1;
		'main: while i < self.transaction_pool.len() {
			println!("checking {} out of {}", i, self.transaction_pool.len());
			'chain: for b in &chain.blocks {
				'trans: for t in &b.data {
					if self.transaction_pool[i] == *t {
						println!("we have a match, this trans shouldnt exist anywhere else");
						self.transaction_pool.remove(i);
						continue 'main;
					}
				}
			}
			i += 1;
		}
	}

/*
	pub fn parse_message(&self, m: Message) {
		if m.form == 0 {
			m.read();
		} else if m.form == 1 {
			let th: rsa::TransHash = bincode::deserialize(&m.mes).unwrap();
			let trans = rsa::verify_transaction(m.user.key.clone(), th);
			rsa::print_transaction(trans);
		}
	}
*/
	pub fn print(&self) {
		for page in &self.book {
			if page.len() > 0 {
				for con in page {
					con.print();
				}
			}
		}
	}
}

fn get_index(name: String) -> usize {
	let mut num = 26;
	match name.chars().next() {
		Some(mut letter) => {
			//let mut letter: char = name.chars().next().unwrap();
			letter.make_ascii_lowercase(); 
			num = letter as usize;
		},
		None => {}
	}
	if num >= 97 && num <= 122 {
		num -= 97;
	} else if num != 26 {
		println!("{} became {}", name, num);
		num = 26;
	}
	num
}

#[derive(Clone, Default)]
pub struct Contact {
	pub key: Vec<u8>,
	pub name: String 
}

impl Contact {
	pub fn new(user: &User) -> Self {
		Self { key: user.public.clone(), name: user.name.clone() }
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let mut v = Vec::new();
		let name_len: u8 = self.name.len().try_into().unwrap();
		v.push(name_len);
		//println!("conatc as bytes:{:#?} len: {}", self.name.clone(), name_len);
		for b in &self.key {
			v.push(*b);
		}
		for b in self.name.as_bytes() {
			v.push(*b);
		}
		v
	}

	pub fn print(&self) {
		println!("name: {}--", self.name);
		println!("key: {}--", String::from_utf8(self.key.clone()).unwrap());
	}

	pub fn save(&self) {
		let b = self.as_bytes();
		match fs::write("contact.sav", b) {
			Ok(_) => {},
			Err(e) => {panic!("failed to save contacts -- {}", e);}
		}
	}
}

pub fn bytes_to_contact(c: Vec<u8>) -> Contact {
	let mut key: Vec<u8> = Vec::new();
	let mut s: Vec<u8> = Vec::new();
	//let name_len = c[272];
	let mut i = 0;
	for b in c {
		if i != 0 {
			if i < 273 {
				key.push(b);
			} else {
				s.push(b);
			}
		}
		i += 1;
	}

	let name = match std::str::from_utf8(&s) {
		Ok(v) => v,
		Err(e) => panic!("bad contact buff {}", e),
	};
	
	Contact {key, name: name.to_string()}	
}
	/*
	for i in 0..272 {
		key.push(c[i]);
	}
	for i in 273..273+name_len {
		s.push(c[i]);
	}
	const name_len: u32 = u32::from_be_bytes(len);
	for i in 272..276 {
		len[i-272] = c[i];
	}
	*/
	/*
	let name: [u8; name_len] = c[276..276+name_len];
	for i in 0..name_len {
		s.push(c[i+276]);
	}
	for b in c {
		if i < 272 {
			key.push(b);
		} else {
			s.push(b);
		}
		i += 1;
	}
	*/
