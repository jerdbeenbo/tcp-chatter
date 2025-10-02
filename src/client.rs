/*
TODO:
    Add a UI with winnit
    Save messages in a database

    egui-winit-wgpu template
    egui examples chat


    could also use this and convert to web assembly to make a web-based chatter, or to send commands up somewhere
*/


use std::io::{BufRead, BufReader, Write};
use std::{
    io::{self, stdin},
    net::TcpStream,
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

fn send_message(msg: String, tcp: &mut TcpStream) -> std::io::Result<()> {
    let formatted = format!("MSG:{}\n", msg.trim());
    tcp.write_all(formatted.as_bytes())?;
    tcp.flush()?;
    Ok(())
}

fn send_session_metadata(name: String, tcp: &mut TcpStream) -> std::io::Result<()> {
    let formatted = format!("META:{}\n", name.trim());
    tcp.write_all(formatted.as_bytes())?;
    tcp.flush()?;
    Ok(())
}

fn get_session_credentials() -> String {
    let mut user_input = String::new();

    println!("Please enter a name for this session: ");
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    user_input
}

///Handles connection to the server, and listen for msg from server
fn handle_connecting(reciever: Receiver<String>) {
    let stream = TcpStream::connect("127.0.0.1:80");

    let mut tcp: TcpStream;
    let mut _continue = false;

    //prompt user to set up credentials
    let session_credentials = get_session_credentials();

    //check to see if connection was successful
    match stream {
        Ok(_) => {
            //It is safe to unwrap value
            tcp = stream.unwrap();


            //send metadata to server to store user information inside of database
            match send_session_metadata(session_credentials, &mut tcp) {
                Ok(_) => println!("-- Credentials registered for this session --"),
                Err(err) => handle_error(err),
            }

            //create thread for handling listening from server now that connection is confirmed
            match tcp.try_clone() {
                Ok(mut cloned) => {
                    thread::spawn(move || {
                        //listen for messages from the server
                        listen_server(&mut cloned); //only need to read, no writing to the stream
                    });
                }
                Err(_) => println!("Could not clone TCP"),
            }

            //listen for input from the user (to send to the server)
            listen_for_user_input(&mut tcp, reciever);
        }
        //connection was unsuccessful
        Err(val) => handle_error(val),
    }
}

fn listen_for_user_input(tcp: &mut TcpStream, reciever: Receiver<String>) {
    //listen_for_user_input for user input
    while true {
        let msg = reciever.recv();

        match msg {
            //msg is all good to send to the server
            Ok(val) => match send_message(val, tcp) {
                Ok(_) => println!("Message Sent"),
                Err(val) => println!("Message failed to send: {}", val),
            },
            //msg is not all good to send to the server
            Err(val) => println!("{}", val),
        }
    }
}

///Reads input from the command line
fn read_input(sender: Sender<String>) {
    print!("λ ");
    let mut input: String = String::new();

    stdin().read_line(&mut input).expect("Unable to read input");

    //trim the input to remove the new line character from read_line
    input = input.trim().to_string();

    sender.send(String::from(input));
}

fn listen_server(tcp: &mut TcpStream) {
    
    let mut reader = BufReader::new(tcp);
    
    while true {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(bytes) => {
                //bytes == 0 means connection closed
                if bytes == 0 {
                    println!("Server disconnected");
                    break;
                }
                else {
                    //got data, process the bytes
                    println!("{}", line.trim());
                }
            },
            Err(_) => {
                println!("Server offline");
                break;
            },
        }
    }
}

///Spawns a thread to listen_for_user_input for messages from the server
///listen_for_user_inputs for user input to send messages to the server on main thread
fn main() {
    //connecting to the server and listen_for_user_inputing for messages on a seperate thread
    let (sender, reciever) = channel::<String>();

    thread::spawn(move || {
        handle_connecting(reciever);
    });

    while true {
        read_input(sender.clone());
    }
}
