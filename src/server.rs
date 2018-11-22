extern crate futures;
extern crate tokio;
extern crate tokio_tungstenite;
extern crate tungstenite;

use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};
use futures::Future;
use fututes::stream::Stream;
use tokio::net::TcpListener;
use tungstenite::protocol::Message;

use tokio_tungstenite::accept_async;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    let socket TcpListner::bind(%addr).unwrap();
    println!("Listnening on: {}", addr);
    
}
