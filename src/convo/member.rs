use std::str::FromStr;
use std::net::{UdpSocket, SocketAddr};

use sdp::{SessionDescription};
use rir::rtp::{RtpSession, RtpPkt, RtpHeader};

pub struct Member {
    pub sdp: SessionDescription,
    pub rtp_session: Option<RtpSession>,
}

impl Member {

    pub fn new(sdp: SessionDescription) -> Member {
        Member {
            sdp: sdp,
            rtp_session: None,
        }
    }

    pub fn init_audio(&mut self) {
        let local_addr =  FromStr::from_str("127.0.0.1").unwrap();
        let bind_socket = SocketAddr::new(local_addr, 0);
        let conn = UdpSocket::bind(bind_socket);

        let rtp_session = new_rtp_session(conn.unwrap(), self.sdp.clone());

        self.rtp_session = Some(rtp_session);
    }

    pub fn write_audio(&self, rtp_pkt: &RtpPkt) {
        debug!("Writing packet to port {}", self.sdp.media.clone().unwrap().port);
        self.rtp_session.as_ref().unwrap().write(rtp_pkt);
    }

    pub fn read_audio(&self) -> RtpPkt {
        let mut rtp_pkt = RtpPkt {
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

        self.rtp_session.as_ref().unwrap().read(&mut rtp_pkt);

        rtp_pkt
    }
}

pub fn new_rtp_session(conn: UdpSocket, sdp: SessionDescription) -> RtpSession {

    let ip_addr = sdp.origin.unwrap().ip_address;
    let port = sdp.media.unwrap().port;

    debug!("Connecting to endpoint {}:{}", ip_addr, port);

    let rtp_stream = RtpSession::connect_to(conn, SocketAddr::new(ip_addr, port));
    
    rtp_stream
}

