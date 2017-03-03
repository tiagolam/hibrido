#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rir;
extern crate opus;
extern crate byteorder;
extern crate hyper;
extern crate rustc_serialize;
extern crate hibrido;

use std::str::from_utf8;
use std::thread;
use std::str::FromStr;
use std::io::prelude::*;
use std::fs::File;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};
use std::net::{TcpStream, UdpSocket, SocketAddr};
use hibrido::sdp::{SessionDescription, Origin};
use hibrido::convo::member::{new_rtp_session};
use std::net::{IpAddr, Ipv6Addr};
use rir::rtp::{RtpSession, RtpPkt, RtpHeader};
use opus::{Decoder, Channels};
use hyper::{Client};
use hyper::status::{StatusCode};
use rustc_serialize::json;
use hibrido::protos::httpserver::{ConferencePost, MemberPost, ConferenceResponse, MemberResponse};

fn convert_i16_to_u8(orig: &mut [i16], dest: &mut [u8]) {
    for i in 0..orig.len() {
        //dest[i] = orig[i*2] as u16;
        //dest[i] = dest[i] << 8;

        dest[i*2] = (orig[i] & 0xFF) as u8;
        dest[i*2 + 1] = ((orig[i] >> 8) & 0xFF) as u8;
        //dest[i] = dest[i] | (orig[i*2 +1] as u8);
    }
}

#[test]
fn test_receive() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    warn!("booting up");

    let local_addr =  FromStr::from_str("127.0.0.1").unwrap();
    let bind_socket = SocketAddr::new(local_addr, 0);
    let conn = UdpSocket::bind(bind_socket).unwrap();

    let local_port = conn.local_addr().unwrap().port();

    let s3_sdp = format!("{} {} {}", "v=0
        o=jdoe 2890844526 2890842807 IN IP4 127.0.0.1
        s=-
        i=A Seminar on the session description protocol
        c=IN IP4 224.2.17.12
        t=0 0
        m=audio", local_port, "RTP/AVP 8 101
        a=rtpmap:8 PCMA/8000
        a=rtpmap:101 opus/48000
        a=ptime:20
        a=recvonly");

    // Create Conference
    let post_body = ConferencePost {
        convo_id: "test_convo_id".to_string(),
    };

    let post_body = json::encode(&post_body).unwrap();
    let client = Client::new();
    let mut res = client.post("http://127.0.0.1:3080/convo")
                        .body(&post_body)
                        .send()
                        .unwrap();

    // If 200, get the :convoid
    if res.status != StatusCode::Ok {
        error!("Bad response {}  when creating conversation", res.status);
        return;
    }
    let ref mut res_body = String::new();
    let _ = res.read_to_string(res_body);
    let convo_res: ConferenceResponse = json::decode(&res_body).unwrap();

    // Create member
    let post_body = MemberPost {
        sdp: s3_sdp,
    };
    let post_body = json::encode(&post_body).unwrap();
    debug!("Body is {}", post_body);
    let post_url = format!("http://127.0.0.1:3080/convo/{}/member", convo_res.convo_id);

    let client = Client::new();
    let mut res = client.post(&post_url)
                        .body(&post_body)
                        .send()
                        .unwrap();

    // If 200, get the :memberid
    if res.status != StatusCode::Ok {
        error!("Bad response {}  when creating member", res.status);
        return;
    }

    let ref mut res_body = String::new();
    let _ = res.read_to_string(res_body);
    let member_res: MemberResponse = json::decode(&res_body).unwrap();

    // read from the socket
    // Get answer
    debug!("\n---received:\n{}", &member_res.sdp);
    let sdp = SessionDescription::new();
    let sdp_answer = sdp.from_sdp(&member_res.sdp);

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

