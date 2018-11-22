extern crate futures;
extern crate tokio;
extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate url;

use std::env;
use std::io::{self, Read, Write};
use std::thread;

use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use tungstenite::protocol::Message;

use tokio_tungstenite::connect_async;

fn main() {
    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = url::Url::parse(&connect_addr).unwrap();

    let (stdin_tx, stdin_rx) = mpsc::channel(0);
    thread::spawn(|| read_stdin(stdin_tx));

    let stdin_rx = stdin_rx.map_err(|_| panic!());

    let mut stdout = io::stdout();

    let client = connect_async(url)
        .and_then(move |(ws_stream, _)| {
            println!("Websocket handshake has been success!");

            let (sink, stream) = ws_stream.split();

            let send_stdin = stdin_rx.forward(sink);
            let write_stdout = stream.for_each(move |message| {
                stdout.write_all(&message.into_data()).unwrap();
                Ok(())
            });

            // Wait for either of futures to complete.
            send_stdin
                .map(|_| ())
                .select(write_stdout.map(|_| ()))
                .then(|_| Ok(()))
        }).map_err(|e| {
            println!("Error during the websocket handshake occurred: {}", e);
            io::Error::new(io::ErrorKind::Other, e)
        });

    // And now that we've got our client, we execute it in the event loop!
    tokio::runtime::run(client.map_err(|_e| ()));
}

fn read_stdin(mut tx: mpsc::Sender<Message>) {
    let mut stdin = io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf) {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx = tx.send(Message::binary(buf)).wait().unwrap();
    }
}
