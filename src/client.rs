use std::{io::{self, stdin}, net::TcpStream, process::exit};

fn handle_connecting() {

}


fn main () {
    
    println!("1. Connect\n2.Exit");
    let mut input: String = String::new();

    stdin().read_line(&mut input).expect("Unable to read input");

    match input {
        "1" => handle_connecting(),
        "2" => exit(0),
        
    }

}