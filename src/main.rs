use std::io::Result;
use std::net::UdpSocket;
use dns_han::handle_query;

fn main() -> Result<()> {

    let socket = UdpSocket::bind(("0.0.0.0",2053))?;

    println!("Server running...");

    loop {
        match handle_query(&socket) {
            Ok(_) => {},
            Err(e) => eprintln!("An error occurred: {}",e),
        }
    }
}


