# dazeus-rs
Rust bindings for DaZeus

## Getting started
Obviously you will need [Rust](http://www.rust-lang.org), at the time of this
writing this means you will need the latest nightlies.

To create a new plugin using these bindings simply run:

    cargo new [plugin-name] --bin

Then in your `Cargo.toml` add:

    [dependencies.dazeus]
    git = "https://github.com/dazeus/dazeus-rs.git"

For parsing command line options I would also suggest you use something like
docopt, to use it, add this to your `Cargo.toml`:

    [dependencies]
    docopt = "0.6"

Then start by using this basic skeleton application in your `main.rs`:

```rust
extern crate dazeus;
extern crate docopt;

use dazeus::{DaZeus, Commander, EventType, connection_from_str};
use docopt::Docopt;

// Write the Docopt usage string.
static USAGE: &'static str = "
A DaZeus plugin.

Usage:
    dazeus-plugin [options]

Options:
    -h, --help                  Show this help message
    -s SOCKET, --socket=SOCKET  Specify the socket DaZeus is listening to, use
                                `unix:/path/to/socket` or `tcp:host:port`
                                [default: unix:/tmp/dazeus.sock]
";

fn main() {
    let args = Docopt::new(USAGE).and_then(|d| d.parse()).unwrap_or_else(|e| e.exit());
    let socket = args.get_str("--socket");

    match connection_from_str(socket) {
        Ok(connection) => {
            let dazeus = DaZeus::from_conn_buff(connection);

            // echo all messages
            dazeus.subscribe(EventType::PrivMsg, |evt| {
                // reply requires an event, and a message (the third event
                // parameter is the message sent to us)
                dazeus.reply(&evt, &evt[3], true);
            });

            // start listening for events
            dazeus.listen();
        },
        Err(e) => {
            println!("Got an error while trying to connect to DaZeus: {}", e);
        }
    }
}
```

Once you have set up your dependencies and created this main file you should be
ready to go using `cargo run`, cargo should install all dependencies, compile
your project and execute the result. If you don't have DaZeus running on your
local machine, or if the default socket location is not what you're looking for,
simply use `cargo run -- --socket=[your socket]`.
