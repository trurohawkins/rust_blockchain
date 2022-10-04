use std::{io, sync::mpsc};

pub fn input(tx: mpsc::Sender<String>) {
	loop {
		//Allow sender t oenter message input
		let mut input = String::new();
		//First access the input message and read it
		io::stdin().read_line(&mut input).expect("Failed to read");
		let _ = tx.send(input);
	}
}
