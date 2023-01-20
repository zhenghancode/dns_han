
pub mod bytesbuffer;
pub mod resultenum;
pub mod dns;

use std::io::{Error,ErrorKind};
use std::net::UdpSocket;
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

        if let Ok(result) = lookup(&question.name, question.qtype) {
            packet.questions.push(question);
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


fn lookup(qname: &str,qtype: QueryType) -> Result<DnsPacket> {

    let server = ("8.8.8.8", 53);
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