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
use byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};
use std::net::{TcpStream, UdpSocket, SocketAddr};
use sdp::{SessionDescription, Origin};
use std::net::{IpAddr, Ipv6Addr};
use convo::member::new_rtp_session;
use rir::rtp::{RtpSession, RtpPkt, RtpHeader};
use opus::{Decoder, Channels};

fn convert_i16_to_u8(orig: &mut [i16], dest: &mut [u8]) {
    for i in 0..orig.len() {
        //dest[i] = orig[i*2] as u16;
        //dest[i] = dest[i] << 8;

        dest[i*2] = (orig[i] & 0xFF) as u8;
        dest[i*2 + 1] = ((orig[i] >> 8) & 0xFF) as u8;
        //dest[i] = dest[i] | (orig[i*2 +1] as u8);
    }
}

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    warn!("booting up");

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

    // 24000 samples / s =  960 bytes per 20ms
    let mut f = File::create("/home/tlam/Downloads/a2002011001-e02-16kHz_dup.wav").unwrap(); 

    // Set up codec decoder
    let mut opus_dec = Decoder::new(16000, Channels::Stereo).unwrap();

    // Set RTP session
    let mut rtp_stream = new_rtp_session(conn, sdp_answer.desc);

    debug!("Ready to receive...");

    loop {
        let mut buffer: [u8; 640] = [0; 640];
        let mut print_buf: Vec<u8> = vec![0; 640];

        // Read
        let rtp_pkt = &mut RtpPkt {
            header: RtpHeader {
                version: 0,
                padding: 0,
                ext: 0,
                cc: 0,
                marker: 0,
                payload_type: 0,
                seq_number: 0,
                timestamp: 0,
                ssrc: 0,
                csrc: vec![],
            },
            payload: vec![],
        };

        rtp_stream.read(rtp_pkt);
        debug!("payload lenght {}", rtp_pkt.payload.len());
        let mut tmp_buffer: Vec<i16> = vec![0; 320];
        let size = opus_dec.decode(&rtp_pkt.payload, &mut tmp_buffer, false).unwrap();
        convert_i16_to_u8(&mut tmp_buffer, &mut buffer);

        debug!("decode size {}", size);
        debug!("buffer lenght {}", buffer.len());
        //convert_i16_to_u8(&mut tmp_buffer, &mut buffer);
        print_buf.clone_from_slice(&buffer);
        debug!("buffer lenght {}", print_buf.len());

        debug!("Decoded after received {:?}", print_buf);

        f.write(&buffer);
    }
}

