/*
    A messenger service that operates over a TCP connection
    Stores every message in a database cause I need more database experience TODO

    Direct message from server to the client
*/

use rusqlite::{Connection, Result};
use axum::{
    extract::{
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//State of the application
struct AppState {
    //Require unique usernames - track in a hashset to ensure no duplicates
    user_set: Mutex<HashSet<String>>,
    
    //Channel used to send messages to all connected clients
    tx: broadcast::Sender<String>,
}

// Simple handler that returns HTML
async fn root_handler() -> Html<&'static str> {
    Html("<h1>Server is running!</h1>")
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

async fn handle_socket(mut socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(write(sender));
    tokio::spawn(read(receiver));
}

async fn read(receiver: SplitStream<WebSocket>) {
    // ...
}

async fn write(sender: SplitSink<WebSocket, Message>) {
    // ...
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
