use std::io::{Read, Write, Result, Error, ErrorKind};
use unix_socket::UnixStream;
use std::net::TcpStream;

/// A connection enum that encapsulates TCP and Unix sockets.
///
/// This enum is mainly used by the `connection_from_str` method. If you want to provide your
/// own connection not retrieved from that function, DaZeus will work with any structure that
/// implements the `std::io::Read` and `std::io::Write` traits.
pub enum Connection {
    /// A Unix domain socket, as implemented by the `unix_socket` crate.
    Unix(UnixStream),
    /// A TCP stream, as implemented by `std::net::TcpStream`.
    Tcp(TcpStream)
}

impl Connection {
    /// Try to duplicate the stream into two objects that reference the same underlying resource.
    pub fn try_clone(&self) -> Result<Connection> {
        match *self {
            Connection::Unix(ref stream) => match stream.try_clone() {
                Ok(cloned) => Ok(Connection::Unix(cloned)),
                Err(e) => Err(e)
            },
            Connection::Tcp(ref stream) => match stream.try_clone() {
                Ok(cloned) => Ok(Connection::Tcp(cloned)),
                Err(e) => Err(e)
            }
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

/// Takes a string in the format type:connection_str and tries to connect
/// to that location. Returns the connection inside an enum that can be used
/// inside DaZeus directly.
pub fn connection_from_str(connection_str: &str) -> Result<Connection> {
    let splits = connection_str.splitn(1, ':').collect::<Vec<_>>();
    match &splits[..] {
        ["unix", path] => {
            match UnixStream::connect(path) {
                Ok(stream) => Ok(Connection::Unix(stream)),
                Err(e) => Err(e)
            }
        },
        ["tcp", location] => {
            match TcpStream::connect(location) {
                Ok(stream) => Ok(Connection::Tcp(stream)),
                Err(e) => Err(e)
            }
        },
        _ => Err(Error::new(ErrorKind::InvalidInput, "Unknown connection type", None))
    }
}
