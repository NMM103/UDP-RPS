#![allow(unused)]
use std::io::{self, prelude::*, BufReader, Read, Write, Error};
use std::str::{self, from_utf8, FromStr};
use rand::{distributions::{Distribution, Standard}, Rng};
use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::*;
use std::thread;
use futures::executor::block_on;

//max length of message
const MAX_MSG_SZ: usize = 50;

#[derive(PartialEq, Debug)]
pub enum Weapon{
	Rock,
	Paper,
	Scissors
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Role{
	Host,
	Client,
}


/**
 *  Randomly generates a weapon. Each outcome has an equal opportunity to happen.
 */
impl Distribution<Weapon> for Standard {
	
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Weapon {
    	match rng.gen_range(0..=2) {
            0 => Weapon::Rock,
            1 => Weapon::Paper,
            _ => Weapon::Scissors,
        }
    }
}

/**
* Turns str into Weapon 
*/
impl FromStr for Weapon {

    type Err = ();

    fn from_str(input: &str) -> Result<Weapon, Self::Err> {
		let tu = input.to_uppercase();
		let trim = tu.trim();
        match trim {
            "ROCK" => Ok(Weapon::Rock),
            "PAPER"  => Ok(Weapon::Paper),
            "SCISSORS"  => Ok(Weapon::Scissors),
            _  => Err(()),
        }
    }
}




fn run_server() -> std::io::Result<()> {
	let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
	let (etx, erx): (Sender<String>, Receiver<String>) = mpsc::channel();

	//creates a UdpSocket here
	let socket = UdpSocket::bind("0.0.0.0:9800")?;

	//new thread to host server
	thread::spawn(move || {
		host(socket, tx, erx);
	});	


	

	//game loop
	loop {
		let msg = rx.recv().unwrap();


		let user_weapon;
		let computer_weapon = rand::random();
		match Weapon::from_str(&msg) {
			Ok(w) => {
				user_weapon = w;
				println!("{}", battle(user_weapon, computer_weapon));
			}
			Err(_e) => {
				let err = String::from("Cannot convert '".to_owned() + &msg + "' to Weapon.");
				etx.send(err);
			}
		};
		
	}
}



/**
 * Loop to host server
 * 
 * socket: socket for UDP connection.
 * tx: Sender<String> to send strings back to main game loop.
 * erx: Reciever<String> used to receive game logic errors to be sent back to user.
 */
fn host(socket: UdpSocket, tx: Sender<String>, erx: Receiver<String>) -> std::io::Result<()>
{
	println!("Server active:");

	loop {
		let mut buf = [0 as u8; MAX_MSG_SZ];
		let (amt, src) = socket.recv_from(&mut buf)?;
		let msg = String::from_utf8((&buf[0..amt-1]).to_vec()).unwrap();


		//send client data through channel
		match tx.send(String::from(msg)){
			Ok(_) => {
				//no issues sending msg
			}
			Err(e) => {
				println!("Error sending message: {}", e)
			}
		}


		//see if there are any messages to send a client
		match erx.try_recv() {
			Ok(msg) => {
				let e = String::from("Error: ".to_owned() + &msg);
				socket.send_to(e.as_bytes(), src);
			}
			Err(e) => {
				//no message in queue, do nothing
			}
		}

	}

	Ok(())
}



/* Battles rps */
fn battle(my_weapon: Weapon, opp_weapon: Weapon) -> String{
	if my_weapon == Weapon::Rock {
		if opp_weapon == Weapon::Rock {
			return String::from("Rock ties Rock");
		} else if opp_weapon == Weapon::Paper {
			return String::from("Rock beats Paper");
		} else {
			return String::from("Rock loses to Scissors");
		}
	} else if my_weapon == Weapon::Paper {
		if opp_weapon == Weapon::Rock {
			return String::from("Paper beats Rock");
		} else if opp_weapon == Weapon::Paper {
			return String::from("Paper ties Paper");
		} else {
			return String::from("Paper loses to Scissors");
		}
	} else {
		if opp_weapon == Weapon::Rock {
			return String::from("Scissors loses to Rock");
		} else if opp_weapon == Weapon::Paper {
			return String::from("Scissors beats Paper");
		} else {
			return String::from("Scissors ties Scissors");
		}
	}
}



fn main() {
	run_server();
}
