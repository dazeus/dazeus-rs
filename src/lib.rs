//! DaZeus IRC bot bindings for [Rust](http://www.rust-lang.org/).
//!
//! For using these bindings you will need to setup a [dazeus-core](https://github.com/dazeus/dazeus-core)
//! instance. For users using OSX and Homebrew, a [tap is available](https://github.com/rnijveld/homebrew-dazeus).
//!
//! The best way to get started is by using the `connection_from_str` function provided. It allows
//! the creation of a `Connection`, which can be fed directly to the `DaZeus::from_conn`
//! constructor.
//!
//! Creating a new connection can now be done using the following basic snippet:
//!
//! ```
//! let dazeus = DaZeus::from_conn(Connection::from_str(socket).unwrap());
//! ```
//!
//! After having created an instance of DaZeus you can start sending commands using one of the
//! methods provided. Alternatively you can send Request objects directly using the
//! `DaZeus::send()` method, however this is generally not recommended.
//!
//! You can register new listeners using the `DaZeus::subscribe()` and
//! `DaZeus::subscribeCommand()` methods. You provide these with functions which will be called
//! every time such an event occurs.
//!
//! After you have enabled any event subscribers you need to use the `DaZeus::listen()` method,
//! or check for new events manually using `DaZeus::try_next_event()`.
//!
//! # Examples
//! The example below creates a simple echo server which responds to some PrivMsg with the exact
//! same reply, only prepending the user that sent the message, so that a highlight is created in
//! IRC clients configured as such.
//!
//! ```
//! let socket = "unix:/tmp/dazeus.sock";
//! let dazeus = DaZeus::from_conn(Connection::from_str(socket).unwrap());
//! dazeus.subscribe(EventType::PrivMsg, |evt, dazeus| {
//!     dazeus.reply(&evt, &evt[3], true);
//! });
//! dazeus.listen();
//! ```
//!
//! The example below creates a connection to the DaZeus server and then immediately joins a
//! channel, and waits for a response until the join was confirmed by the DaZeus core. Note how
//! this is just a short-run command, in contrast to the previous example that will keep running
//! for as long as it can.
//!
//! ```
//! let socket = "unix:/tmp/dazeus.sock";
//! let dazeus = DaZeus::from_conn(Connection::from_str(socket).unwrap());
//! dazeus.join("local", "#test");
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
