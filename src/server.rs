/*
    A messenger service that operates over a TCP connection
    Stores every message in a database cause I need more database experience TODO

    Direct message from server to the client
*/

use rusqlite::{Connection, Result};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use axum::{
    routing::{get, post},
    response::Html,
    Json, Router,
};
use serde::{Deserialize, Serialize};

// Simple handler that returns HTML
async fn root_handler() -> Html<&'static str> {
    Html("<h1>Server is running!</h1>")
}

///Handles a client connection to the mutual exhange
///Connects a user to the database
fn handle_connection(
    mut stream: TcpStream,
    clients_clone: Arc<Mutex<Vec<TcpStream>>>,
    conn: Arc<Mutex<Connection>>,
) {
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
                    if line.trim().starts_with("META:") {

                        println!("line starts with meta");

                        //store the data in a database
                        let db_conn = conn.lock().unwrap();
                        let timestamp = chrono::Utc::now();

                        let content = line.trim().strip_prefix("META:").unwrap();

                        match store_metadata(
                            &db_conn,
                            &peer_addr.to_string(),
                            content,
                            timestamp,
                        ) {
                            Ok(_) => {
                                //store message
                                println!("Metadata for {:?} stored", peer_addr);
                            }
                            Err(e) => println!("{}", e),
                        }

                        continue;
                    }

                    //got data (message from another client)
                    //store message in database
                    //relay message to other connected user
                    let content = line.trim().strip_prefix("MSG:").unwrap();

                    println!("message {}", content);

                    println!("Message from {:?}.\nMessage: {}", stream_clone, content);

                    //store the data in a database
                    let db_conn = conn.lock().unwrap();
                    let timestamp = chrono::Utc::now();
                    match store_message(&db_conn, &peer_addr.to_string(), content, timestamp) {
                        Ok(_) => {
                            //store message
                            println!("Message stored");
                        }
                        Err(e) => println!("{}", e),
                    }

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
                //Disconnect the client gracefully

                //specify immutable list of current clients list
                let mut client_list = clients_clone.lock().unwrap();

                for i in 0..client_list.len() {
                    if client_list[i].peer_addr().unwrap() == peer_addr {
                        client_list.remove(i);
                        break; //This is important!
                    }
                }

                println!("Clients list size is now: {:?}", client_list.len());

                println!("Client disconnected {:?}", peer_addr);
                break;
            }
        }
    }
}

fn store_metadata(
    conn: &Connection,
    sender: &str,
    content: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    //attempt to store the message in the table
    conn.execute(
        "INSERT INTO user (senderADR, username, timestamp) VALUES (?1, ?2, ?3)",
        (sender, content, timestamp.to_string()),
    )?;

    Ok(())
}

fn store_message(
    conn: &Connection,
    sender: &str,
    content: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    //attempt to store the message in the table
    conn.execute(
        "INSERT INTO messages (senderADR, content, timestamp) VALUES (?1, ?2, ?3)",
        (sender, content, timestamp.to_string()),
    )?;

    Ok(())
}

fn handle_db() -> Result<Connection, rusqlite::Error> {
    let conn: Connection = Connection::open("chatlog.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
        senderADR TEXT PRIMARY KEY,
        username TEXT,
        timestamp TEXT NOT NULL
    )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY,
        senderADR TEXT NOT NULL,
        content TEXT NOT NULL,
        timestamp TEXT NOT NULL,
        FOREIGN KEY (senderADR) REFERENCES user(senderADR)
    )",
        [],
    )?;

    Ok(conn) //Return the database connection
}

// Run TCP server in a blocking thread
fn run_tcp_server(conn: Arc<Mutex<Connection>>) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;
    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    println!("TCP server listening on 127.0.0.1:80");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                clients.lock().unwrap().push(stream.try_clone()?);
                let clients_clone = Arc::clone(&clients);
                let conn_clone = Arc::clone(&conn);
                thread::spawn(move || {
                    handle_connection(stream, clients_clone, conn_clone);
                });
            }
            Err(_e) => {
                println!("Connection failed");
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialise database
    let conn: Arc<Mutex<Connection>> = Arc::new(Mutex::new(handle_db().unwrap()));

    // Clone connection for TCP server
    let tcp_conn = Arc::clone(&conn);

    // Spawn TCP server in a blocking thread pool
    tokio::task::spawn_blocking(move || {
        if let Err(e) = run_tcp_server(tcp_conn) {
            eprintln!("TCP server error: {}", e);
        }
    });

    // Build Axum app
    let app = Router::new()
        .route("/", get(root_handler));

    // Bind and serve Axum
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5500").await.unwrap();
    println!("Axum server listening on 127.0.0.1:5500");
    
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

/*

Thread 1: conn_clone1 ----\
                           \
Thread 2: conn_clone2 -------> [Same Database in Memory]
                           /
Thread 3: conn_clone3 ----/

How Arc cloning works

*/
