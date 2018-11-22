extern crate futures;
extern crate tokio;
extern crate tokio_tungstenite;
extern crate tungstenite;

use futures::stream::Stream;
use futures::Future;
use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
//use std::net::TcpListener;

use tungstenite::protocol::Message;

use tokio_tungstenite::accept_async;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listnening on: {}", addr);

    let connections = Arc::new(Mutex::new(HashMap::new()));

    let srv = socket.incoming().for_each(move |stream| {
        let addr = stream
            .peer_addr()
            .expect("Connected streams should have a peer address");

        let connections_inner = connections.clone();

        accept_async(stream)
            .and_then(move |ws_stream| {
                println!("New websocket connection: {}", addr);

                let (tx, rx) = futures::sync::mpsc::unbounded();
                connections_inner.lock().unwrap().insert(addr, tx);

                let (sink, stream) = ws_stream.split();
                let connections = connections_inner.clone();

                let ws_reader = stream.for_each(move |message: Message| {
                    println!("Received a message from {}: {}", addr, message);

                    let mut conns = connections.lock().unwrap();
                    let iter = conns
                        .iter_mut()
                        .filter(|&(&k, _)| k != addr)
                        .map(|(_, v)| v);
                    for tx in iter {
                        tx.unbounded_send(message.clone()).unwrap();
                    }
                    Ok(())
                });

                // Whenever we receive a string on the Receiver, we write it to
                // `WriteHalf<WebSocketStream>`.
                let ws_writer = rx.fold(sink, |mut sink, msg| {
                    use futures::Sink;
                    sink.start_send(msg).unwrap();
                    Ok(sink)
                });
                // Now that we've got futures representing each half of the socket, we
                // use the `select` combinator to wait for either half to be done to
                // tear down the other. Then we spawn off the result.
                let connection = ws_reader
                    .map(|_| ())
                    .map_err(|_| ())
                    .select(ws_writer.map(|_| ()).map_err(|_| ()));

                tokio::spawn(connection.then(move |_| {
                    connections_inner.lock().unwrap().remove(&addr);
                    println!("Connection {} closed.", addr);
                    Ok(())
                }));

                Ok(())
            }).map_err(|e| {
                println!("Error during the websocket handshake occurred: {}", e);
                Error::new(ErrorKind::Other, e)
            })
    });

    // Execute server.
    tokio::runtime::run(srv.map_err(|_e| ()));
}
