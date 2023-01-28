
pub mod bytesbuffer;
pub mod resultenum;
pub mod dns;

use std::io::{Error,ErrorKind};
use std::net::{UdpSocket,Ipv4Addr};
use crate::resultenum::ResultCode;
use crate::{dns::{DnsPacket,QueryType,DnsQuestion},bytesbuffer::BytePacketBuffer};
use std::io::Result;

pub fn default_error() -> Error {
    Error::new(ErrorKind::Other, "End of buffer")
}


pub fn error_with_jumps(max:i32) -> Error {
    Error::new(ErrorKind::Other,format!("Limit of {} jumps exceeded", max))
}

pub fn error_for_custom(msg: &str) -> Error {
    Error::new(ErrorKind::Other,msg)
}

pub fn handle_query(socket: &UdpSocket) -> Result<()> {

    let mut req_buffer = BytePacketBuffer::new();

    let (_,src) = socket.recv_from(&mut req_buffer.buf)?;

    let mut request = DnsPacket::from_buffer(&mut req_buffer)?;

    let mut packet = DnsPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.response = true;

    if let Some(question) = request.questions.pop() {
        println!("Received query: {:?}",question);

        if let Ok(result) = recursive_lookup(&question.name, question.qtype) {
            packet.questions.push(question.clone());
            packet.header.rescode = result.header.rescode;

            for rec in result.answers {
                println!("Answer: {:?}",rec);
                packet.answers.push(rec);
            }

            for rec in result.authorities {
                println!("Answer: {:?}",rec);
                packet.authorities.push(rec);
            }

            for rec in result.resources {
                println!("Answer: {:?}",rec);
                packet.resources.push(rec);
            }
        } else {
            packet.header.rescode = ResultCode::SERVFAIL;
        }
    } else {
        packet.header.rescode = ResultCode::FORMERR;
    }

    let mut res_buffer = BytePacketBuffer::new();
    packet.write(&mut res_buffer)?;

    let len = res_buffer.pos();
    let data = res_buffer.get_range(0,len)?;

    socket.send_to(data, src)?;

    Ok(())
}


fn lookup(qname: &str,qtype: QueryType,server: (Ipv4Addr,u16)) -> Result<DnsPacket> {

    // let server = ("8.8.8.8", 53);
    let socket = UdpSocket::bind(("0.0.0.0",43210))?;
    let mut packet = DnsPacket::new();

    packet.header.id = 6666;
    packet.header.questions =1;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(DnsQuestion::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;
    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf)?;

    DnsPacket::from_buffer(&mut res_buffer)
}

fn recursive_lookup(qname: &str,qtype: QueryType) -> Result<DnsPacket> {

    // For now we're always starting with *a.root-servers.net*.
    let mut ns = "198.41.0.4".parse::<Ipv4Addr>().unwrap();

    loop {
        println!("attempting lookup of {:?} {} with ns {}",qtype,qname,ns);

        let ns_copy = ns;

        let server = (ns_copy,53);
        let response = lookup(qname, qtype,server)?;

        if !response.answers.is_empty() && response.header.rescode == ResultCode::NOERROR {
            return Ok(response);
        }

        // We might also get a `NXDOMAIN` reply, which is the authoritative name servers
        // way of telling us that the name doesn't exist.
        if response.header.rescode == ResultCode::NXDOMAIN {
            return Ok(response);
        }

        if let Some(new_ns) = response.get_resolved_ns(qname) {
            ns = new_ns;
            
            continue;
        }

        let new_ns_name = match response.get_unresolved_ns(qname) {
            Some(x) => x,
            None => return Ok(response),
        };

        let recursive_response = recursive_lookup(new_ns_name, QueryType::A)?;

        if let Some(new_ns) = recursive_response.get_random_a() {
            ns = new_ns;
        } else {
            return Ok(response);
        }
    }
}