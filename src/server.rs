/*
    A messenger service that operates over a TCP connection
    Stores every message in a database cause I need more database experience TODO

    Direct message from server to the client
*/

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

///Handles a client connection to the mutual exhange
///Connects a user to the database
fn handle_connection(mut stream: TcpStream, clients_clone: Arc<Mutex<Vec<TcpStream>>>) {
    println!("Client connected on: {:?}", stream);

    let stream_clone = stream.try_clone(); //for printing
    let peer_addr = stream.peer_addr().unwrap(); //to avoid ownership issues in if comparision

    let mut reader = BufReader::new(stream);

    while true {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(bytes) => {
                //bytes == 0 means connection closed
                if bytes == 0 {
                    println!("Client disconnected");
                    break;
                } else {
                    //got data (message from another client)
                    //store message in database
                    //relay message to other connected user
                    println!("Message from {:?}.\nMessage: {}", stream_clone, line.trim());

                    //store the data in a database

                    //relay the message to the other client
                    //lock the contact list
                    let mut client_list = clients_clone.lock().unwrap();

                    let client_amt = client_list.len();

                    //only one person connencted (let them know no one is getting the messages)
                    if client_amt == 1 {
                        //reader.get_mut() is just the client tcpstream "stream". but reader holds ownership
                        match reader
                            .get_mut()
                            .write_all("No one is listening right now\n".as_bytes())
                        {
                            Ok(_) => println!("Relayed to user no one else is active"),
                            Err(val) => println!("{}", val),
                        }
                    }

                    //send a message to all clients
                    for client in client_list.iter_mut() {
                        //dont send the message to youself
                        if client.peer_addr().unwrap() == peer_addr {
                            continue;
                        }

                        match client.write_all(line.as_bytes()) {
                            Ok(_) => println!("Message sent to {:?}", client),
                            Err(_) => println!("Message failed to send"),
                        }
                    }
                }
            }
            Err(_) => {
                println!("Client disconnected");
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;

    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                //push stream to shared contacts list
                clients.lock().unwrap().push(stream.try_clone()?);
                //clone contacts list
                let clients_clone = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_connection(stream, clients_clone);
                });
            }
            Err(e) => {
                println!("Connection failed");
            }
        }
    }
    Ok(())
}
