/*
    A messenger service that operates over a TCP connection
    Stores every message in a database cause I need more database experience

    Direct message from server to the client

    Store message in database
*/

use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::thread;

///Handles a client connection to the mutual exhange
///Connects a user to the database
fn handle_connection(stream: TcpStream) {

    println!("Client connected on: {:?}", stream);
    
    let mut reader = BufReader::new(stream);
    
    while true {
        /*
           tcp.read() or BufReader::new(tcp).read_line()
           How to continuously read incoming messages
           Message parsing from byte streams

           Thread communication:
           How does listen_server() display received messages without interfering with the input prompt?
        */
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(bytes) => {
                //bytes == 0 means connection closed
                if bytes == 0 {
                    println!("Client disconnected");
                    break;
                }
                else {
                    //got data, process the bytes
                    println!("{}", line.trim());
                }
            },
            Err(_) => println!("An error occured"),
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {println!("Connection failed");}
        }
    }
    Ok(())
}
