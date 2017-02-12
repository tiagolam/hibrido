#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rir;
#[macro_use(lazy_static, __lazy_static_create)]
extern crate lazy_static;
extern crate opus;
extern crate byteorder;

pub mod sdp;
mod convo;

use std::str::from_utf8;
use std::thread;
use std::str::FromStr;
use std::io::prelude::*;
use std::fs::File;
use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::net::{TcpStream, UdpSocket, SocketAddr};
use sdp::{SessionDescription, Origin};
use std::net::{IpAddr, Ipv6Addr};
use convo::member::new_rtp_session;
use rir::rtp::{RtpSession, RtpPkt, RtpHeader};
use opus::{Encoder, Application, Channels, Error};

fn convert_u8_to_i16(orig: &mut [u8], dest: &mut [i16]) {

    for i in 0..dest.len() {
        dest[i] = LittleEndian::read_i16(&[orig[i], orig[i+1]]);
    }
}

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    warn!("booting up");

    let mut s1 = SessionDescription::new();
    s1.ver = Some(1);
    s1.origin = Some(Origin {
        username: "me".to_string(),
        session_id: "sessA".to_string(),
        session_version: 11,
        net_type: sdp::NetType::IN,
        addr_type: sdp::AddrType::IP4,
        ip_address: IpAddr::V6(Ipv6Addr::new(0,0,0,0,0,0xffff,5,2))
    });
    let s1_exp = format!("{:?}", s1);

    let s2_res = s1.from_sdp(&s1_exp);

    let local_addr =  FromStr::from_str("127.0.0.1").unwrap();
    let bind_socket = SocketAddr::new(local_addr, 0);
    let conn = UdpSocket::bind(bind_socket).unwrap();

    let local_port = conn.local_addr().unwrap().port();

    let s3_sdp = format!("{} {} {}", "conference_id: conference_test_id
        sdp: v=0
        o=jdoe 2890844526 2890842807 IN IP4 127.0.0.1
        s=SDP Seminar
        i=A Seminar on the session description protocol
        c=IN IP4 224.2.17.12
        t=2873397496 2873404696
        a=recvonly
        m=audio", local_port, "RTP/AVP 0");

    // Send SDP using TCP
    let mut stream = TcpStream::connect("127.0.0.1:30000").unwrap();
    let _ = stream.write(s3_sdp.as_bytes());
    stream.flush();

    // read from the socket
    // Get answer
    let mut buf = [0; 1500];
    let _ = stream.read(&mut buf);
    println!("\n---received:\n{}", from_utf8(&buf).unwrap());
    let sdp = SessionDescription::new();
    let sdp_answer = sdp.from_sdp(from_utf8(&buf).unwrap());

    // 16000 samples / s =  640 bytes per 20ms
    let mut f = File::open("/home/tlam/Downloads/a2002011001-e02-16kHz.wav").unwrap(); 

    // Set up codec
    let mut opus_enc = Encoder::new(16000, Channels::Stereo, Application::Audio).unwrap();

    // Set RTP session
    let mut rtp_stream = new_rtp_session(conn, sdp_answer.desc);

    // Write
    let mut rtp_pkt = RtpPkt {
        header: RtpHeader {
            version: 2,
            padding: 0,
            ext: 0,
            cc: 0,
            marker: 0,
            payload_type: 0,
            seq_number: 0,
            timestamp: 0,
            ssrc: 0123456789,
            csrc: vec![],
        }, 
        payload: vec![],
    };

    let mut buffer: [u8; 640] = [0; 640];
    let mut tmp_buffer: [i16; 320] = [0; 320];
    let mut encoded: [u8; 640] = [0; 640];
    let mut seq = 0;
    let mut ts = 0;

    while let Ok(x) = f.read(&mut buffer) {
        if x == 0 {
            break;
        }

        let mut print_buf: Vec<u8> = vec![];
        for i in 0..buffer.len() {
            print_buf.push(buffer[i]);
        }
        debug!("Decoded before sent is {:?}", print_buf);

        rtp_pkt.header.seq_number = seq;
        rtp_pkt.header.timestamp = ts;

        convert_u8_to_i16(&mut buffer, &mut tmp_buffer);

        let size = opus_enc.encode(&tmp_buffer, &mut encoded).unwrap();

        rtp_pkt.payload = vec![0; size];
        rtp_pkt.payload.clone_from_slice(&encoded[..size]);

        debug!("Writing packet with payload of size {}, seq {} and ts {}", size, seq, ts);

        rtp_stream.write(&rtp_pkt);
        thread::sleep_ms(20);

        ts += 20;
        if seq >= (65535 - 1) {
            seq = 0;
        } else {
            seq += 1;
        }
    }
}

