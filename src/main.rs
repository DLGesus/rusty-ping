extern crate clap;
extern crate pnet;

use clap::{App, Arg};
use pnet::transport::{transport_channel, TransportChannelType::Layer3};
use pnet::transport::{ipv4_packet_iter, icmpv6_packet_iter};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::echo_reply::EchoReplyPacket;
use pnet::packet::icmp::{IcmpTypes, IcmpCode, checksum, IcmpPacket};
use pnet::packet::icmpv6::{Icmpv6Types, Icmpv6Code, MutableIcmpv6Packet};
use pnet::packet::ipv4::{Ipv4Packet, checksum as ipv4_checksum};
use pnet::packet::{Packet, MutablePacket};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::thread;

fn main() {
    let matches = App::new("rusty-ping")
                    .version("1.0")
                    .about("Sample Text")
                    .author("Chris DeLaGarza")
                    .arg(Arg::with_name("IP")
                        .help("IPv4 or IPv6 address to ping")
                        .required(true)
                        .index(1))
                    .arg(Arg::with_name("TTL")
                        .long("ttl")
                        .help("Set the time to live field for the package")
                        .takes_value(true))
                    .get_matches();
    
    let ip_str = matches.value_of("IP").expect("IP field address not specified");
    let mut addrs_iter = format!("{}:443", ip_str).to_socket_addrs().unwrap();
    let ipaddr = addrs_iter.next().unwrap().ip();
    let ttl = matches.value_of("TTL").unwrap_or("128");
    let ttl: u8 = ttl.parse().unwrap();

    match ipaddr {
        IpAddr::V4(ip) => {
            v4_ping(ip, ttl);
        },
        IpAddr::V6(ip) => {
            v6_ping(ip, ttl);
        }
    }

}

fn v4_ping(ip: Ipv4Addr, mut ttl: u8) {
    // Setup the channel for sending and accepting icmp packages
    let (mut sender, mut receiver) = transport_channel(4096, Layer3(IpNextHeaderProtocols::Icmp)).unwrap();
    let mut iter = ipv4_packet_iter(&mut receiver);
    let mut sequence_number = 0;
    let mut total_sent = 0;
    let mut total_sucessful = 0;
    loop {
        let ip_buf: &mut [u8] = &mut [0; 28];
        let icmp_buf: &mut [u8] = &mut [0; 8];
        let ping_packet = build_v4_packet(ip_buf, icmp_buf, ip, sequence_number, ttl);

        println!("Pinging {} with TTL={}", ip, ping_packet.get_ttl());
        sender.send_to(ping_packet, IpAddr::V4(ip)).unwrap();
        let now = Instant::now();
        total_sent += 1;
        // Get the incoming packet
        let timeout = Duration::from_secs(2);
        loop {
            if let Some((packet, addr)) = iter.next_with_timeout(timeout).unwrap() {
                let elasped = now.elapsed().as_micros();
                let elasped_str = match elasped < 1000 {
                    true => format!("<1"),
                    false => format!("{}", elasped / 1000)
                };
                let icmp_packet = EchoReplyPacket::new(packet.payload()).unwrap();
                if icmp_packet.get_sequence_number() == sequence_number {
                    if icmp_packet.get_icmp_type() == IcmpTypes::EchoReply {
                        total_sucessful += 1;
                        println!("Reply from {}: bytes={} time={}ms TTL={}", addr, packet.get_total_length(), elasped_str, packet.get_ttl());
                    }
                    else if icmp_packet.get_icmp_type() == IcmpTypes::TimeExceeded {
                        println!("Packet Error: Time exceeded.");
                        ttl += 1;
                    }
                    else {
                        println!("Received reply, but unhandled message");
                    }
                    sequence_number += 1;
                    break;
                }
            }
            else {
                println!("Packet timed out");
                ttl += 1;
                break;
            }
        }
        println!("\tPackets Sent = {}, Received = {}, Lost = {} ({}% loss)\n", 
                    total_sent, total_sucessful, total_sent - total_sucessful, 100 * (total_sent - total_sucessful) / total_sent);
        thread::sleep(match Duration::from_secs(2) <= now.elapsed() {
            true => Duration::from_secs(0),
            false => Duration::from_secs(2) - now.elapsed()
        });

    }
}

fn v6_ping(ip: Ipv6Addr, ttl: u8) {
    // Setup the channel for sending and accepting icmp packages
    let (mut sender, mut receiver) = transport_channel(4096, Layer3(IpNextHeaderProtocols::Icmpv6)).unwrap();
    let mut iter = icmpv6_packet_iter(&mut receiver);
    let mut sequence_number = 0;
    let mut total_sent = 0;
    let mut total_sucessful = 0;
    loop {
        let ip_buf: &mut [u8] = &mut [0; 48];
        let icmp_buf: &mut [u8] = &mut [0; 8];
        let ping_packet = build_v6_packet(ip_buf, icmp_buf, ip, sequence_number, ttl);

        println!("Pinging {}", ip);
        if let Err(_) = sender.send_to(ping_packet, IpAddr::V6(ip)) {
            panic!("Ipv6 do not seem to be very supported by the library pnet. But if this line would work the code should work fine.");
        }
        let now = Instant::now();
        total_sent += 1;
        // Get the incoming packet
        let timeout = Duration::from_secs(2);
        loop {
            if let Some((packet, addr)) = iter.next_with_timeout(timeout).unwrap() {
                let elasped = now.elapsed().as_micros();
                let elasped_str = match elasped < 1000 {
                    true => format!("<1"),
                    false => format!("{}", elasped / 1000)
                };
                let icmp_packet = EchoReplyPacket::new(packet.payload()).unwrap();
                if icmp_packet.get_sequence_number() == sequence_number {
                    if packet.get_icmpv6_type() == Icmpv6Types::EchoReply {
                        total_sucessful += 1;
                        println!("Reply from {}:\ttime={}ms", addr, elasped_str);
                    }
                    else if packet.get_icmpv6_type() == Icmpv6Types::TimeExceeded {
                        println!("Packet Error: Time exceeded.")
                    }
                    else {
                        println!("Received reply, but unhandled message");
                    }
                    sequence_number += 1;
                    break;
                }
            }
            else {
                println!("Packet timed out");
                break;
            }
        }
        println!("\tPackets Sent = {}, Received = {}, Lost = {} ({}% loss)", 
                    total_sent, total_sucessful, total_sent - total_sucessful, 100 * (total_sent - total_sucessful) / total_sent);
        thread::sleep(match Duration::from_secs(2) <= now.elapsed() {
            true => Duration::from_secs(0),
            false => Duration::from_secs(2) - now.elapsed()
        });

    }
}

fn build_icmp_packet<'x>(icmp_buffer: &'x mut [u8], sequence_number: u16, is_v4: bool) -> MutableEchoRequestPacket<'x> {
    let mut icmp_packet = MutableEchoRequestPacket::new(icmp_buffer).unwrap();
    icmp_packet.set_sequence_number(sequence_number);
    icmp_packet.set_identifier(44); // Magic Number
    if is_v4 {
        icmp_packet.set_icmp_type(IcmpTypes::EchoRequest); // Type = 0 (Echo Request)
        icmp_packet.set_icmp_code(IcmpCode::new(0));
    }
    else {
        let mut icmpv6_packet = MutableIcmpv6Packet::new(icmp_packet.packet_mut()).unwrap();
        icmpv6_packet.set_icmpv6_type(Icmpv6Types::EchoRequest); // Type = 128 (Echo Request)
        icmpv6_packet.set_icmpv6_code(Icmpv6Code::new(0));
    }
    let icmp_checksum = checksum(&IcmpPacket::new(icmp_packet.packet()).unwrap());
    icmp_packet.set_checksum(icmp_checksum);
    icmp_packet
}

fn build_v4_packet<'x>(ip_buffer: &'x mut [u8], icmp_buffer: &'x mut [u8], dest: Ipv4Addr, sequence_number: u16, ttl:u8) -> MutableIpv4Packet<'x> {
    // Build ip packet
    let mut ip_packet = MutableIpv4Packet::new(ip_buffer).unwrap();
    ip_packet.set_version(4);
    ip_packet.set_header_length(5);
    ip_packet.set_total_length(28);
    ip_packet.set_identification(44);
    ip_packet.set_ttl(ttl);
    ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
    ip_packet.set_destination(dest);
    let ip_checksum = ipv4_checksum(&Ipv4Packet::new(ip_packet.packet()).unwrap());
    ip_packet.set_checksum(ip_checksum);

    // Build the icmp packet
    let mut icmp_packet = build_icmp_packet(icmp_buffer, sequence_number, true);

    // Place the icmp packet into the ip packets's payload
    ip_packet.set_payload(icmp_packet.packet_mut());
    ip_packet
}

fn build_v6_packet<'x>(ip_buffer: &'x mut [u8], icmp_buffer: &'x mut [u8], dest: Ipv6Addr, sequence_number: u16, ttl: u8) -> MutableIpv6Packet<'x> {
    // Build ip packet
    let mut ip_packet = MutableIpv6Packet::new(ip_buffer).unwrap();
    ip_packet.set_version(6);
    ip_packet.set_payload_length(8);
    ip_packet.set_next_header(IpNextHeaderProtocols::Icmpv6);
    ip_packet.set_hop_limit(ttl);
    ip_packet.set_destination(dest);

    // Build the icmp packet
    let mut icmp_packet = build_icmp_packet(icmp_buffer, sequence_number, false);

    // Place the icmp packet into the ip packets's payload
    ip_packet.set_payload(icmp_packet.packet_mut());
    ip_packet
}
