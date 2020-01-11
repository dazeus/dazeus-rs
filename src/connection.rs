use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::TcpStream;
use std::str::FromStr;
use unix_socket::UnixStream;

/// A connection enum that encapsulates TCP and Unix sockets.
///
/// This enum is mainly used by the `connection_from_str` method. If you want to provide your
/// own connection not retrieved from that function, DaZeus will work with any structure that
/// implements the `std::io::Read` and `std::io::Write` traits.
pub enum Connection {
    /// A Unix domain socket, as implemented by the `unix_socket` crate.
    Unix(UnixStream),
    /// A TCP stream, as implemented by `std::net::TcpStream`.
    Tcp(TcpStream),
}

impl Connection {
    /// Try to duplicate the stream into two objects that reference the same underlying resource.
    pub fn try_clone(&self) -> Result<Connection> {
        match *self {
            Connection::Unix(ref stream) => match stream.try_clone() {
                Ok(cloned) => Ok(Connection::Unix(cloned)),
                Err(e) => Err(e),
            },
            Connection::Tcp(ref stream) => match stream.try_clone() {
                Ok(cloned) => Ok(Connection::Tcp(cloned)),
                Err(e) => Err(e),
            },
        }
    }
}

impl FromStr for Connection {
    type Err = Error;

    /// Takes a string in the format type:connection_str and tries to connect
    /// to that location. Returns the connection inside an enum that can be used
    /// inside DaZeus directly.
    fn from_str(connection_str: &str) -> Result<Self> {
        let splits = connection_str.splitn(2, ':').collect::<Vec<_>>();
        if splits.len() == 2 && splits[0] == "unix" {
            Ok(Connection::Unix(UnixStream::connect(splits[1])?))
        } else if splits.len() == 2 && splits[0] == "tcp" {
            Ok(Connection::Tcp(TcpStream::connect(splits[1])?))
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "Unknown connection type",
            ))
        }
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match *self {
            Connection::Unix(ref mut stream) => stream.read(buf),
            Connection::Tcp(ref mut stream) => stream.read(buf),
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match *self {
            Connection::Unix(ref mut stream) => stream.write(buf),
            Connection::Tcp(ref mut stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match *self {
            Connection::Unix(ref mut stream) => stream.flush(),
            Connection::Tcp(ref mut stream) => stream.flush(),
        }
    }
}
