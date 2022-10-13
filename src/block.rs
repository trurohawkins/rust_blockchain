use serde::{Serialize, Deserialize};
use std::fs;
use log::{warn, error};
use chrono::prelude::*;//{Duration, Utc};
use sha2::{Digest, Sha256};
use crate::{rsa};

const DIFFICULTY_PREFIX: &str = "0";

fn hash_to_binary_representation(hash: &[u8]) -> String {
	let mut res: String = String::default();
	for c in hash {
		res.push_str(&format!("{:b}", c));
	}
	res
}

fn calculate_hash<T: Serialize>(id: u64, timestamp: i64, previous_hash: &str, data: T, nonce: u64) -> Vec<u8> {
	let data = serde_json::json!({
		"id": id,
		"previous_hash": previous_hash,
		"data": data,
		"timestamp": timestamp,
		"nonce": nonce
	});
	let mut hasher = Sha256::new();
	hasher.update(data.to_string().as_bytes());
	hasher.finalize().as_slice().to_owned()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
	pub id: u64,
	pub hash: String,
	pub previous_hash: String,
	pub timestamp: i64,
	pub data: Vec<rsa::Transaction>,
	pub nonce: u64
}

impl Block {
	pub fn new(id: u64, previous_hash: String, data: Vec<rsa::Transaction>) -> Self {
		let now = Utc::now();
		let (nonce, hash) = mine_block(id, now.timestamp(), &previous_hash, &data);
		Self {
			id, hash, timestamp: now.timestamp(), previous_hash, data, nonce,
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chain {
	pub blocks: Vec<Block>
}

impl Chain {
	pub fn new() -> Self {
		let blocks = load_chain();
		let mut chain = Chain {
			blocks: blocks
		};
		if chain.blocks.len() == 0 {
			chain.genesis(Vec::new());
		}
		chain
	}

	pub fn genesis(&mut self, data: Vec<rsa::Transaction>) {
		let genesis_block = Block {
			id: 0,
			timestamp: Utc::now().timestamp(),
			previous_hash: String::from("genesis"),
			data: data, //String::from("genesis!"),
			nonce: 28366,
			hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(), 
		};
		self.blocks.push(genesis_block);
	}

	pub fn try_add_block(&mut self, block: Block) -> bool {
		let latest_block = self.blocks.last().expect("theereis at least one block");
		if self.is_block_valid(&block, latest_block) {
			self.blocks.push(block);
			true
		} else {
			error!("could not add block - invalid");
			false
		}
	}

	fn is_block_valid (&self, block: &Block, previous_block: &Block) -> bool {
		if block.previous_hash != previous_block.hash {
			warn!("block with id : {} has wrong previous hash", block.id);
			return false;
		} else if !hash_to_binary_representation (
			&hex::decode(&block.hash).expect("can decode from hex"),
		).starts_with(DIFFICULTY_PREFIX)
		{
			warn!("block with id: {} has invalid difficulty", block.id);
			return false;
		} else if block.id != previous_block.id + 1 {
			warn!(
				"block with is: {} is not the nmext block fter the latest: {}",
				block.id, previous_block.id
			);
			return false;
		} else if hex::encode(calculate_hash(
			block.id,
			block.timestamp,
			&block.previous_hash,
			&block.data,
			block.nonce
		)) != block.hash {
			warn!("block with id: {} has invaluid hash", block.id);
			return false;
		}
		true
	}

	pub fn is_chain_valid(&self, chain: &[Block]) -> bool {
		for i in 0..chain.len() {
			if i == 0 {
				continue;
			}
			let first = chain.get(i - 1).expect("has to exist");
			let second =  chain.get(i).expect("has to exist");
			if !self.is_block_valid(second, first) {
				return false;
			}
		}
		true
	}

	// We always choose the longest chain
	pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
		let is_local_valid = self.is_chain_valid(&local);
		let is_remote_valid = self.is_chain_valid(&remote);

		if is_local_valid && is_remote_valid {
			if local.len() >= remote.len() {
				local
			} else {
				remote
			}
		} else if is_remote_valid && !is_local_valid {
			remote
		} else if !is_remote_valid && is_local_valid {
			local
		} else {
			panic!("LOCAL AND REMOTE CHAINS ARE BOTH INVALID!");
		}
	}

	pub fn save_chain(&mut self) {
		/*
		let mut buffer: Vec<u8> = Vec::new();
		for b in &self.blocks {
			let d = bincode::serialize(b).unwrap();
			for byte in d {
				buffer.push(byte);
			}
		}
		*/
		let buffer = bincode::serialize(&self.blocks).unwrap();
		match fs::write("blockchain.sav", buffer) {
			Ok(_) => {},
			Err(e) => {panic!("failed to save blockchain -- {}", e);}
		}
	}
}

pub fn load_chain() -> Vec<Block> {
	//let  mut  data: 'de;
	match fs::read("blockchain.sav") {
		Ok(d) => {
			let chain = bincode::deserialize(&d).unwrap();
			return chain;
		},
		Err(e) => {
			println!("couldn't load blockchain {}", e);
			Vec::new()
		}
	}
}

pub fn mine_block<T: Serialize + Copy>(id: u64, timestamp: i64, previous_hash: &str, data: T) -> (u64, String) {
	println!("mining block..");
	let mut nonce = 0;
	
	loop {
		if nonce % 100000 == 0 {
			println!("nonce: {}", nonce);
		}
		let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
		let binary_hash = hash_to_binary_representation(&hash);
		if binary_hash.starts_with(DIFFICULTY_PREFIX) {
			println!(
				"mined! nonce {}\n hash: {}\n binary hash: {}",
				nonce,
				hex::encode(&hash),
				binary_hash
			);
			return (nonce, hex::encode(hash));
		}
		nonce += 1;
	}
}

pub fn print_transaction_block(b: &Block) {
	println!("~~BLOCK # {}~~", b.id);
	println!("   hash: {}", b.hash);
	println!("   nonce: {}", b.nonce);
	println!("   pre: {}", b.previous_hash);
	println!("   at {}", b.timestamp);
	for t in &b.data {
		rsa::print_transaction(&t);
	}
}
pub fn chain_print_transactions(chain: &Chain) {
	for b in &chain.blocks {
		print_transaction_block(&b);
	}
}
