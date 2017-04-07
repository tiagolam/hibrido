extern crate uuid;

use lazy_static;
use self::uuid::Uuid;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::net::{UdpSocket, SocketAddr};

use sdp::{SessionDescription};
use rir::rtp::{RtpSession, RtpPkt, RtpHeader};
use ice;

// TODO(tlam): There's possible high contention here, as all member creations
// pass through here
lazy_static! {
    static ref members_by_id: Mutex<HashMap<String, Arc<Mutex<Member>>>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
}

pub struct Member {
    pub id: String,
    pub sdp: SessionDescription,
    pub rtp_session: Option<RtpSession>,
    pub ice: ice::Ice,
}

impl Member {

    pub fn new(sdp: SessionDescription) -> Arc<Mutex<Member>> {
        let member_id: &str = &Uuid::new_v4().to_string();

        debug!("Creating a new member [{}]", member_id);

        let member = Member {
            id: member_id.to_string(),
            sdp: sdp,
            rtp_session: None,
        };

        unsafe {
            members_by_id.lock().unwrap().insert(member_id.to_string(), Arc::new(Mutex::new(member)));
            return members_by_id.lock().unwrap().get(member_id).unwrap().clone();
        }
    }

    pub fn get(id: &str) -> Option<Arc<Mutex<Member>>> {
        unsafe {
            if members_by_id.lock().unwrap().contains_key(id) {
                return Some(members_by_id.lock().unwrap().get(id).unwrap().clone());
            } else {
                return None;
            }
        }
    }

    pub fn start_session(&self, &base_sdp) {
    }

    pub fn init_audio(&mut self) {
        let local_addr =  FromStr::from_str("10.0.0.138"/*"192.168.2.186"*/).unwrap();
        let bind_socket = SocketAddr::new(local_addr, 6000);
        let conn = UdpSocket::bind(bind_socket);

        let rtp_session = new_rtp_session(conn.unwrap(), self.sdp.clone());

        self.rtp_session = Some(rtp_session);
    }

    pub fn write_audio(&self, rtp_pkt: &RtpPkt) {
        debug!("Writing packet to port {}", self.sdp.media[0].clone().media.port);
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
    let port = sdp.media[0].media.port;

    debug!("Connecting to endpoint {}:{}", ip_addr, port);

    let rtp_stream = RtpSession::connect_to(conn, SocketAddr::new(ip_addr, port));
    
    rtp_stream
}

