use core::error;
use std::{error::{self, Error}, fs::read, io::{self, stdin}, net::TcpStream, process::exit, thread};
use std::sync::Mutex;

fn handle_connecting() -> Result<TcpStream, error> {
    
    let k = TcpStream::connect("127.0.0.1:80");
}



fn read_input() {
    println!("λ ");
    let mut input: String = String::new();

    stdin().read_line(&mut input).expect("Unable to read input");
}


fn main () {
    
    //connecting to the server and listening for messages on a seperate thread
    thread::spawn(|| {
        handle_connecting();
    });

    while true {
        read_input();
    }
}