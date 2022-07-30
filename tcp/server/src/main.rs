use std::net::{TcpListener, TcpStream};
use std::io::{Error, Read, Write};
use std::thread;
use std::str;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind( "0.0.0.0:3456" )?;
    println!( "listening [{}] ...", listener.local_addr()? );

    for streams in listener.incoming() {
        match streams {
            Err( e ) => { eprintln!( "error: {}", e ) },
            Ok( stream ) => {
                thread::spawn( move || {
                    handler( stream ).unwrap_or_else( |error| eprintln!( "{:?}", error ) );
                });
            }
        }
    }

    Ok(())
}

fn handler( mut stream: TcpStream ) -> Result<(), Error> {
    println!( "Connect from {}", stream.peer_addr()? );
    let mut buf = [0; 1024];
    loop {
        let nbytes = stream.read( &mut buf )?;
        if nbytes == 0 {
            return Ok(());
        }

        let str = str::from_utf8( &buf[..nbytes] ).unwrap();
        println!( "received: [{}]", str );

        stream.write( &buf[..nbytes] )?;
        stream.flush()?;
        println!( "send: [{}]", str );
    }
}