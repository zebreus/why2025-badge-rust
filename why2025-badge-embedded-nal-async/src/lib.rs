#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod implementation {
    use embedded_nal_async::TcpConnect;

    pub use std_embedded_nal_async::Stack as BadgeDns;
    pub use std_embedded_nal_async::Stack as BadgeTcpConnect;

    pub type BadgeTcpConnection<'a> = <BadgeTcpConnect as TcpConnect>::Connection<'a>;
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
mod implementation {
    use std::fmt;
    use std::future::Future;
    use std::io;
    use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
    use std::pin::Pin;
    use std::sync::{Arc, Mutex, MutexGuard};
    use std::task::{Context, Poll, Waker};
    use std::thread;

    use embedded_io_async::{ErrorType, Read, Write};
    use embedded_nal_async::{AddrType, Dns, TcpConnect};

    #[derive(Clone, Copy, Debug, Default)]
    pub struct BadgeDns;

    impl BadgeDns {
        pub const fn new() -> Self {
            Self
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct BadgeTcpConnect;

    impl BadgeTcpConnect {
        pub const fn new() -> Self {
            Self
        }
    }

    pub type BadgeTcpConnection<'a> = BadgeTcpConnectionInner;

    #[derive(Debug)]
    pub struct BadgeTcpConnectionInner {
        stream: Arc<Mutex<TcpStream>>,
    }

    #[derive(Debug)]
    struct BlockingTask<T> {
        shared: Arc<Mutex<BlockingTaskState<T>>>,
    }

    #[derive(Debug)]
    struct BlockingTaskState<T> {
        result: Option<io::Result<T>>,
        waker: Option<Waker>,
    }

    #[derive(Debug)]
    struct ReadOutcome {
        buffer: Vec<u8>,
        len: usize,
    }

    fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn spawn_blocking<T, F>(name: &'static str, operation: F) -> io::Result<BlockingTask<T>>
    where
        T: Send + 'static,
        F: FnOnce() -> io::Result<T> + Send + 'static,
    {
        let shared = Arc::new(Mutex::new(BlockingTaskState {
            result: None,
            waker: None,
        }));
        let worker_shared = Arc::clone(&shared);

        thread::Builder::new().name(name.into()).spawn(move || {
            let result = operation();
            let waker = {
                let mut state = lock_unpoisoned(&worker_shared);
                state.result = Some(result);
                state.waker.take()
            };

            if let Some(waker) = waker {
                waker.wake();
            }
        })?;

        Ok(BlockingTask { shared })
    }

    fn resolve_host(host: String, addr_type: AddrType) -> io::Result<IpAddr> {
        for addr in (host.as_str(), 0).to_socket_addrs()? {
            let ip = addr.ip();
            match (&addr_type, ip) {
                (AddrType::Either, _) => return Ok(ip),
                (AddrType::IPv4, IpAddr::V4(_)) => return Ok(ip),
                (AddrType::IPv6, IpAddr::V6(_)) => return Ok(ip),
                _ => {}
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            format!("no address matched requested type for {host}"),
        ))
    }

    impl<T> Future for BlockingTask<T> {
        type Output = io::Result<T>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut state = lock_unpoisoned(&self.shared);
            if let Some(result) = state.result.take() {
                Poll::Ready(result)
            } else {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    impl TcpConnect for BadgeTcpConnect {
        type Error = io::Error;
        type Connection<'a>
            = BadgeTcpConnection<'a>
        where
            Self: 'a;

        async fn connect<'a>(
            &'a self,
            remote: SocketAddr,
        ) -> Result<Self::Connection<'a>, Self::Error> {
            let stream = spawn_blocking("badge-net-connect", move || TcpStream::connect(remote))
                ?.await?;
            Ok(BadgeTcpConnectionInner {
                stream: Arc::new(Mutex::new(stream)),
            })
        }
    }

    impl Dns for BadgeDns {
        type Error = io::Error;

        async fn get_host_by_name(
            &self,
            host: &str,
            addr_type: AddrType,
        ) -> Result<IpAddr, Self::Error> {
            let host = host.to_owned();
            spawn_blocking("badge-dns-lookup", move || resolve_host(host, addr_type))?.await
        }

        async fn get_host_by_address(
            &self,
            addr: IpAddr,
            result: &mut [u8],
        ) -> Result<usize, Self::Error> {
            let text = addr.to_string();
            if result.len() < text.len() {
                return Err(io::Error::other("buffer too small for host string"));
            }

            result[..text.len()].copy_from_slice(text.as_bytes());
            Ok(text.len())
        }
    }

    impl ErrorType for BadgeTcpConnectionInner {
        type Error = io::Error;
    }

    impl Read for BadgeTcpConnectionInner {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let stream = Arc::clone(&self.stream);
            let read_len = buf.len();
            let outcome = spawn_blocking("badge-net-read", move || {
                let mut read_buffer = vec![0_u8; read_len];
                let len = {
                    let mut stream = lock_unpoisoned(&stream);
                    std::io::Read::read(&mut *stream, &mut read_buffer)?
                };

                Ok(ReadOutcome {
                    buffer: read_buffer,
                    len,
                })
            })?
            .await?;

            buf[..outcome.len].copy_from_slice(&outcome.buffer[..outcome.len]);
            Ok(outcome.len)
        }
    }

    impl Write for BadgeTcpConnectionInner {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            let stream = Arc::clone(&self.stream);
            let write_buffer = buf.to_vec();
            spawn_blocking("badge-net-write", move || {
                let mut stream = lock_unpoisoned(&stream);
                std::io::Write::write(&mut *stream, &write_buffer)
            })?
            .await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            let stream = Arc::clone(&self.stream);
            spawn_blocking("badge-net-flush", move || {
                let mut stream = lock_unpoisoned(&stream);
                std::io::Write::flush(&mut *stream)
            })?
            .await
        }
    }

    impl fmt::Display for BadgeTcpConnectionInner {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "BadgeTcpConnection")
        }
    }
}

pub use implementation::*;

#[cfg(test)]
mod tests {
    use std::io::{Read as _, Write as _};
    use std::net::IpAddr;
    use std::net::TcpListener;
    use std::thread;

    use embedded_io_async::{Read as _, Write as _};
    use embedded_nal_async::{Dns as _, TcpConnect as _};

    use super::*;

    #[test]
    fn reverse_lookup_returns_text() {
        let dns = BadgeDns::default();
        let mut buf = [0_u8; 64];
        let length = futures::executor::block_on(
            dns.get_host_by_address(IpAddr::from([127, 0, 0, 1]), &mut buf),
        )
        .unwrap();

        assert!(length > 0);
        assert!(std::str::from_utf8(&buf[..length]).is_ok());
    }

    #[test]
    fn tcp_connection_round_trips_io() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(&request, b"ping");
            stream.write_all(b"pong").unwrap();
            stream.flush().unwrap();
        });

        let connector = BadgeTcpConnect::default();
        let mut connection = futures::executor::block_on(connector.connect(address)).unwrap();

        futures::executor::block_on(connection.write_all(b"ping")).unwrap();
        futures::executor::block_on(connection.flush()).unwrap();

        let mut response = [0_u8; 4];
        futures::executor::block_on(connection.read_exact(&mut response)).unwrap();
        assert_eq!(&response, b"pong");

        server.join().unwrap();
    }
}