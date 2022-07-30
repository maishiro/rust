use std::io::prelude::*;
use std::net::TcpStream;
use std::str;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3456")?;

    let str = "test";
    let buf = str.as_bytes();
    stream.write( &buf )?;
    println!( "send: [{}]", str );

    let mut buf = [0; 1024];
    let len = stream.read( &mut buf )?;
    let str = str::from_utf8( &buf[..len] ).unwrap();
    println!( "received: [{}]", str );

    Ok(())
}
