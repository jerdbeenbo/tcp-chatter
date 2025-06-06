use core::error;
use std::io::Write;
use std::sync::*;
use std::{
    error::{self, Error},
    fs::read,
    io::{self, stdin},
    net::TcpStream,
    process::exit,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

///Handle the error and print the appropraite coresspondance
fn handle_error(err: io::Error) {
    match err.kind() {
        io::ErrorKind::NotConnected => println!("Not connected to the server"),
        io::ErrorKind::ConnectionRefused => println!("Connection refused"),
        io::ErrorKind::TimedOut => println!("Connection timed out"),
        _ => println!("An unexpected network error occurred: {:?}", err),
    }
}

///Sends the message that was collected from the command line to the server
fn send_message(msg: String, tcp: &mut TcpStream) -> std::io::Result<()> {
    
    //Send the msg as bytes
    tcp.write_all(msg.as_bytes())?;


    Ok(())
}

///Handles connection to the server, and listens for msg from server
fn handle_connecting(reciever: Receiver<String>) {
    let stream = TcpStream::connect("127.0.0.1:80");

    let mut tcp: TcpStream;
    let mut _continue = false;

    //check to see if connection was successful
    match stream {
        Ok(_) => {
            //It is safe to unwrap value
            tcp = stream.unwrap();
            listen(&mut tcp, reciever);
        }
        //connection was unsuccessful
        Err(val) => handle_error(val),
    }
}

fn listen(tcp: &mut TcpStream, reciever: Receiver<String>) {
    //listen for user input
    while true {
        let msg = reciever.recv();

        match msg {
            //msg is all good to send to the server
            Ok(val) => {
                match send_message(val, tcp) {
                    Ok(_) => println!("Message Sent"),
                    Err(val) => println!("Message failed to send: {}", val),
                }
            },
            //msg is not all good to send to the server
            Err(val) => println!("{}", val),
        }
    }
}

///Reads input from the command line
fn read_input(sender: Sender<String>) {
    println!("λ ");
    let mut input: String = String::new();

    stdin().read_line(&mut input).expect("Unable to read input");

    //trim the input to remove the new line character from read_line
    input = input.trim().to_string();

    sender.send(String::from(input));
}

///Spawns a thread to listen for messages from the server
///Listens for user input to send messages to the server on main thread
fn main() {
    //connecting to the server and listening for messages on a seperate thread
    let (sender, reciever) = channel::<String>();
    thread::spawn(move || {
        handle_connecting(reciever);
    });


    while true {
        read_input(sender.clone());
    }
}
