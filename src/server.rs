/*
    A messenger service that operates over a TCP connection
    Stores every message in a database cause I need more database experience
*/

use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;

///Handles a client connection to the mutual exhange
///Connects a user to the database
fn handle_connection(stream: TcpStream) {
   println!("client connected on {:?}", stream);
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {println!("Connection failed");}
        }
    }
    Ok(())
}
