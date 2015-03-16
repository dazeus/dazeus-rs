# dazeus-rs
Rust bindings for DaZeus

## Getting started
Obviously you will need [Rust][http://www.rust-lang.org], at the time of this
writing this means you will need the latest nightlies, but as soon as 1.0 is
available, a stable version should work just fine as well.

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

use docopt::Docopt;
use dazeus::DaZeus;
use dazeus::util::connection_from_str;

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

            // set up some listeners here

            dazeus.listen();
        },
        Err(e) => {
            println!("Got an error while trying to connect to DaZeus: {}", e);
        }
    }
}
```
