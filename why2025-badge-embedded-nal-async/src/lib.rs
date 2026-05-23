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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, MutexGuard};
    use std::task::{Context, Poll, Waker};
    use std::thread::{self, JoinHandle, Thread};

    use embedded_io_async::{ErrorType, Read, Write};
    use embedded_nal_async::{AddrType, Dns, TcpConnect};

    static NEXT_CONNECTION_ID: AtomicUsize = AtomicUsize::new(1);

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
        connection_id: usize,
        remote: SocketAddr,
        worker: Option<ConnectionWorker>,
    }

    #[derive(Debug)]
    struct ConnectionWorker {
        connection_id: usize,
        remote: SocketAddr,
        shared: Arc<Mutex<ConnectionWorkerState>>,
        thread: Thread,
        join_handle: Option<JoinHandle<()>>,
    }

    #[derive(Debug, Default)]
    struct ConnectionWorkerState {
        pending_command: Option<ConnectionCommand>,
        closed: bool,
    }

    #[derive(Debug)]
    enum ConnectionCommand {
        Read {
            len: usize,
            response: TaskCompleter<ReadOutcome>,
        },
        Write {
            buffer: Vec<u8>,
            response: TaskCompleter<usize>,
        },
        Flush {
            response: TaskCompleter<()>,
        },
        Close,
    }

    #[derive(Debug)]
    struct TaskFuture<T> {
        shared: Arc<Mutex<TaskState<T>>>,
    }

    #[derive(Debug)]
    struct TaskCompleter<T> {
        shared: Arc<Mutex<TaskState<T>>>,
    }

    #[derive(Debug)]
    struct TaskState<T> {
        result: Option<io::Result<T>>,
        waker: Option<Waker>,
    }

    #[derive(Debug)]
    struct ReadOutcome {
        buffer: Vec<u8>,
    }

    fn addr_type_name(addr_type: &AddrType) -> &'static str {
        match addr_type {
            AddrType::Either => "either",
            AddrType::IPv4 => "ipv4",
            AddrType::IPv6 => "ipv6",
        }
    }

    fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn next_connection_id() -> usize {
        NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed)
    }

    fn task_completion<T>() -> (TaskFuture<T>, TaskCompleter<T>) {
        let shared = Arc::new(Mutex::new(TaskState {
            result: None,
            waker: None,
        }));

        (
            TaskFuture {
                shared: Arc::clone(&shared),
            },
            TaskCompleter { shared },
        )
    }

    impl<T> TaskCompleter<T> {
        fn complete(self, result: io::Result<T>) {
            let waker = {
                let mut state = lock_unpoisoned(&self.shared);
                state.result = Some(result);
                state.waker.take()
            };

            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    fn spawn_blocking<T, F>(name: &'static str, operation: F) -> io::Result<TaskFuture<T>>
    where
        T: Send + 'static,
        F: FnOnce() -> io::Result<T> + Send + 'static,
    {
        let (future, completer) = task_completion();

        println!("[badge-nal] scheduling worker name={name}");

        match thread::Builder::new().name(name.into()).spawn(move || {
            println!("[badge-nal] worker started name={name}");
            let result = operation();
            match &result {
                Ok(_) => println!("[badge-nal] worker completed name={name}"),
                Err(error) => println!("[badge-nal] worker failed name={name} error={error}"),
            }

            completer.complete(result);
        }) {
            Ok(_) => {}
            Err(error) => {
                println!("[badge-nal] failed to spawn worker name={name} error={error}");
                return Err(error);
            }
        }

        Ok(future)
    }

    fn resolve_host(host: String, addr_type: AddrType) -> io::Result<IpAddr> {
        println!(
            "[badge-nal] dns lookup start host={host} addr_type={}",
            addr_type_name(&addr_type)
        );

        let addrs = match (host.as_str(), 0).to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(error) => {
                println!("[badge-nal] dns lookup failed host={host} error={error}");
                return Err(error);
            }
        };

        for addr in addrs {
            let ip = addr.ip();
            println!("[badge-nal] dns candidate host={host} candidate={ip}");
            match (&addr_type, ip) {
                (AddrType::Either, _) => {
                    println!("[badge-nal] dns selected host={host} ip={ip}");
                    return Ok(ip);
                }
                (AddrType::IPv4, IpAddr::V4(_)) => {
                    println!("[badge-nal] dns selected host={host} ip={ip}");
                    return Ok(ip);
                }
                (AddrType::IPv6, IpAddr::V6(_)) => {
                    println!("[badge-nal] dns selected host={host} ip={ip}");
                    return Ok(ip);
                }
                _ => {}
            }
        }

        let error = io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            format!("no address matched requested type for {host}"),
        );
        println!(
            "[badge-nal] dns lookup no-match host={host} addr_type={} error={error}",
            addr_type_name(&addr_type)
        );
        Err(error)
    }

    fn connection_closed_error(connection_id: usize, remote: SocketAddr) -> io::Error {
        io::Error::new(
            io::ErrorKind::BrokenPipe,
            format!("badge connection worker exited connection_id={connection_id} remote={remote}"),
        )
    }

    fn run_connection_worker(
        connection_id: usize,
        remote: SocketAddr,
        shared: Arc<Mutex<ConnectionWorkerState>>,
        ready: TaskCompleter<()>,
    ) {
        println!(
            "[badge-nal] worker started name=badge-net-connection connection_id={connection_id} remote={remote}"
        );

        let mut stream = match TcpStream::connect(remote) {
            Ok(stream) => {
                match stream.local_addr() {
                    Ok(local_addr) => println!(
                        "[badge-nal] tcp connect success remote={remote} local={local_addr} connection_id={connection_id}"
                    ),
                    Err(error) => println!(
                        "[badge-nal] tcp connect success remote={remote} local=<unavailable> error={error} connection_id={connection_id}"
                    ),
                }
                ready.complete(Ok(()));
                stream
            }
            Err(error) => {
                println!(
                    "[badge-nal] tcp connect failed remote={remote} error={error} connection_id={connection_id}"
                );
                ready.complete(Err(error));
                println!(
                    "[badge-nal] worker completed name=badge-net-connection connection_id={connection_id} remote={remote}"
                );
                return;
            }
        };

        loop {
            let command = loop {
                let mut state = lock_unpoisoned(&shared);
                if let Some(command) = state.pending_command.take() {
                    break Some(command);
                }

                if state.closed {
                    break None;
                }

                drop(state);
                thread::park();
            };

            let Some(command) = command else {
                break;
            };

            match command {
                ConnectionCommand::Read { len, response } => {
                    println!(
                        "[badge-nal] read start requested_len={len} connection_id={connection_id}"
                    );
                    let mut buffer = vec![0_u8; len];
                    let result = std::io::Read::read(&mut stream, &mut buffer).map(|read| {
                        buffer.truncate(read);
                        ReadOutcome { buffer }
                    });

                    match &result {
                        Ok(outcome) => println!(
                            "[badge-nal] read success requested_len={len} actual_len={} connection_id={connection_id}",
                            outcome.buffer.len()
                        ),
                        Err(error) => println!(
                            "[badge-nal] read failed requested_len={len} error={error} connection_id={connection_id}"
                        ),
                    }

                    response.complete(result);
                }
                ConnectionCommand::Write { buffer, response } => {
                    let requested_len = buffer.len();
                    println!(
                        "[badge-nal] write start requested_len={requested_len} connection_id={connection_id}"
                    );
                    let result = std::io::Write::write(&mut stream, &buffer);

                    match &result {
                        Ok(written) => println!(
                            "[badge-nal] write success requested_len={requested_len} actual_len={written} connection_id={connection_id}"
                        ),
                        Err(error) => println!(
                            "[badge-nal] write failed requested_len={requested_len} error={error} connection_id={connection_id}"
                        ),
                    }

                    response.complete(result);
                }
                ConnectionCommand::Flush { response } => {
                    println!("[badge-nal] flush start connection_id={connection_id}");
                    let result = std::io::Write::flush(&mut stream);

                    match &result {
                        Ok(()) => {
                            println!("[badge-nal] flush success connection_id={connection_id}")
                        }
                        Err(error) => println!(
                            "[badge-nal] flush failed error={error} connection_id={connection_id}"
                        ),
                    }

                    response.complete(result);
                }
                ConnectionCommand::Close => break,
            }
        }

        println!(
            "[badge-nal] worker completed name=badge-net-connection connection_id={connection_id} remote={remote}"
        );
    }

    impl ConnectionWorker {
        fn submit(&self, command: ConnectionCommand) -> io::Result<()> {
            {
                let mut state = lock_unpoisoned(&self.shared);
                if state.closed {
                    return Err(connection_closed_error(self.connection_id, self.remote));
                }

                if state.pending_command.is_some() {
                    return Err(io::Error::other(format!(
                        "badge connection already has a pending command connection_id={} remote={}",
                        self.connection_id, self.remote
                    )));
                }

                state.pending_command = Some(command);
            }

            self.thread.unpark();
            Ok(())
        }

        fn shutdown(&mut self) {
            println!(
                "[badge-nal] connection worker shutdown requested connection_id={} remote={}",
                self.connection_id, self.remote
            );

            {
                let mut state = lock_unpoisoned(&self.shared);
                state.closed = true;
                if state.pending_command.is_none() {
                    state.pending_command = Some(ConnectionCommand::Close);
                }
            }

            self.thread.unpark();

            if let Some(join_handle) = self.join_handle.take() {
                match join_handle.join() {
                    Ok(()) => println!(
                        "[badge-nal] connection worker joined connection_id={} remote={}",
                        self.connection_id, self.remote
                    ),
                    Err(_) => println!(
                        "[badge-nal] connection worker panicked connection_id={} remote={}",
                        self.connection_id, self.remote
                    ),
                }
            }
        }
    }

    impl<T> Future for TaskFuture<T> {
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
            let connection_id = next_connection_id();
            let (ready_future, ready) = task_completion();
            let shared = Arc::new(Mutex::new(ConnectionWorkerState::default()));
            let worker_shared = Arc::clone(&shared);

            println!("[badge-nal] tcp connect start remote={remote} connection_id={connection_id}");
            println!(
                "[badge-nal] scheduling worker name=badge-net-connection connection_id={connection_id} remote={remote}"
            );

            let join_handle = match thread::Builder::new()
                .name(format!("badge-net-connection-{connection_id}"))
                .spawn(move || run_connection_worker(connection_id, remote, worker_shared, ready))
            {
                Ok(join_handle) => join_handle,
                Err(error) => {
                    println!(
                        "[badge-nal] failed to spawn worker name=badge-net-connection error={error} connection_id={connection_id} remote={remote}"
                    );
                    return Err(error);
                }
            };

            let worker_thread = join_handle.thread().clone();

            match ready_future.await {
                Ok(()) => Ok(BadgeTcpConnectionInner {
                    connection_id,
                    remote,
                    worker: Some(ConnectionWorker {
                        connection_id,
                        remote,
                        shared,
                        thread: worker_thread,
                        join_handle: Some(join_handle),
                    }),
                }),
                Err(error) => {
                    {
                        let mut state = lock_unpoisoned(&shared);
                        state.closed = true;
                    }
                    worker_thread.unpark();
                    match join_handle.join() {
                        Ok(()) => {}
                        Err(_) => println!(
                            "[badge-nal] connection worker panicked during connect connection_id={connection_id} remote={remote}"
                        ),
                    }
                    Err(error)
                }
            }
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
            match spawn_blocking("badge-dns-lookup", move || resolve_host(host, addr_type))?.await {
                Ok(ip) => {
                    println!("[badge-nal] dns lookup success ip={ip}");
                    Ok(ip)
                }
                Err(error) => {
                    println!("[badge-nal] dns lookup task failed error={error}");
                    Err(error)
                }
            }
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

    impl BadgeTcpConnectionInner {
        fn worker_mut(&mut self) -> io::Result<&mut ConnectionWorker> {
            self.worker
                .as_mut()
                .ok_or_else(|| connection_closed_error(self.connection_id, self.remote))
        }
    }

    impl Read for BadgeTcpConnectionInner {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let read_len = buf.len();
            let (future, response) = task_completion();
            self.worker_mut()?.submit(ConnectionCommand::Read {
                len: read_len,
                response,
            })?;

            let outcome = future.await?;
            let actual_len = outcome.buffer.len();
            buf[..actual_len].copy_from_slice(&outcome.buffer);
            Ok(actual_len)
        }
    }

    impl Write for BadgeTcpConnectionInner {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            let (future, response) = task_completion();
            self.worker_mut()?.submit(ConnectionCommand::Write {
                buffer: buf.to_vec(),
                response,
            })?;

            future.await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            let (future, response) = task_completion();
            self.worker_mut()?
                .submit(ConnectionCommand::Flush { response })?;

            future.await
        }
    }

    impl Drop for BadgeTcpConnectionInner {
        fn drop(&mut self) {
            if let Some(mut worker) = self.worker.take() {
                worker.shutdown();
            }
        }
    }

    impl fmt::Display for BadgeTcpConnectionInner {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "BadgeTcpConnection({})", self.connection_id)
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
