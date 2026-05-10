use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
};

fn main() -> std::io::Result<()> {
    let Some(address) = env::args().nth(1) else {
        println!("usage: std-tcp-demo host:port");
        return Ok(());
    };

    println!("connecting to {address}");
    let mut stream = TcpStream::connect(address)?;
    stream.write_all(b"ping from BadgeVMS std\n")?;

    let mut buffer = [0u8; 128];
    match stream.read(&mut buffer) {
        Ok(0) => println!("server closed connection"),
        Ok(n) => println!("received: {}", String::from_utf8_lossy(&buffer[..n])),
        Err(error) => println!("read failed after write: {error}"),
    }

    Ok(())
}
