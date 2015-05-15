//! DaZeus IRC bot bindings for [Rust](http://www.rust-lang.org/).
//!
//! For using these bindings you will need to setup a [dazeus-core](https://github.com/dazeus/dazeus-core)
//! instance. For users using OSX and Homebrew, a [tap is available](https://github.com/rnijveld/homebrew-dazeus).
//!
//! The best way to get started is by using the `connection_from_str` function provided. It allows
//! the creation of a `Connection`, which can be fed directly to the `DaZeus::from_conn`
//! constructor.
//!
//! For many connections it is recommended to use a buffered Read+Write. This allows the stream
//! to flush a whole Json object (no matter the size) at once using an internal buffer, instead of
//! depending on the underlying system. To use a buffered connection with a `Connection` object,
//! use the `DaZeus::from_conn_buff` constructor.
//!
//! Creating a new connection can now be done using the following basic snippet:
//!
//! ```
//! match connection_from_str(socket) {
//!     Ok(connection) => {
//!         let dazeus = DaZeus::from_conn_buff(connection);
//!     },
//!     Err(e) => {
//!         println!("Got an error while trying to connect to DaZeus: {}", e);
//!     }
//! }
//! ```
//!
//! After having created an instance of DaZeus you can start sending commands using one of the
//! methods provided in the `Commander` trait. Alternatively you can send Request objects directly
//! using the `DaZeus::send()` method, however this is generally not recommended.
//!
//! You can register new listeners using the `Commander::subscribe()` and
//! `Commander::SubscribeCommand()` methods. You provide these with functions which will be called
//! every time such an event occurs.
//!
//! After you have enabled any event subscribers you need to use the `DaZeus::listen()` method,
//! or check for new events manually using either the blocking `DaZeus::next_event()` or the
//! non-blocking `DaZeus::try_next_event()`.
//!
//! # Examples
//! The example below creates a simple echo server which responds to some PrivMsg with the exact
//! same reply, only prepending the user that sent the message, so that a highlight is created in
//! IRC clients configured as such.
//!
//! ```
//! let socket = "unix:/tmp/dazeus.sock";
//! match connection_from_str(socket) {
//!     Ok(connection) => {
//!         let dazeus = DaZeus::from_conn_buff(connection);
//!         dazeus.subscribe(EventType::PrivMsg, |evt| {
//!             dazeus.reply(&evt, &evt[3], true);
//!         });
//!         dazeus.listen();
//!     },
//!     Err(e) => println!("Could not connect to DaZeus: {}", e);
//! }
//! ```
//!
//! The example below creates a connection to the DaZeus server and then immediately joins a
//! channel, and waits for a response until the join was confirmed by the DaZeus core. Note how
//! this is just a short-run command, in contrast to the previous example that will keep running
//! for as long as it can.
//!
//! ```
//! let socket = "unix:/tmp/dazeus.sock";
//! match connection_from_str(socket) {
//!     Ok(connection) => {
//!         let dazeus = DaZeus::from_conn_buff(connection);
//!         dazeus.join("local", "#test").into_inner();
//!     },
//!     Err(e) => println!("Could not connect to DaZeus: {}", e);
//! }
//! ```

#[macro_use]
extern crate log;
extern crate rustc_serialize as serialize;
extern crate unix_socket;

pub use self::connection::*;
pub use self::dazeus::*;
pub use self::error::*;
pub use self::event::*;
pub use self::listener::ListenerHandle;
pub use self::request::*;
pub use self::response::*;
pub use self::scope::*;

mod connection;
mod dazeus;
mod event;
mod handler;
mod listener;
mod request;
mod response;
mod scope;
mod error;
