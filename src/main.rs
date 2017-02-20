#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rir;
#[macro_use(lazy_static, __lazy_static_create)]
extern crate lazy_static;
extern crate opus;
#[macro_use]
extern crate nickel;
extern crate rustc_serialize;

mod sdp;
mod protos;
mod convo;

use sdp::{SessionDescription, Origin};
use std::net::{IpAddr, Ipv6Addr};
use protos::handlers;

fn main() {

    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("Firing up!...");

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

    let s3_res = s1.from_sdp("
        v=0
        o=jdoe 2890844526 2890842807 IN IP4 10.47.16.5
        s=-
        i=A Seminar on the session description protocol
        c=IN IP4 224.2.17.12/127
        t=0 0
        a=recvonly
        m=audio 49170 RTP/AVP 0
    ");

    println!("---\nv3_res:\n{:?}", s3_res);

    // Start TCP server and listen for SDPs
    //let tcp_server = protos::tcpserver::tcp::new();
    //tcp_server.start_server();
    //protos::tcpserver::tcp::start_server();
    protos::httpserver::HttpServer::start_server();
}

