use lazy_static;
use std::collections::HashMap;
use std::boxed::Box;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use convo::member::Member;
use sdp::{SessionDescription};
use std::net::{UdpSocket, SocketAddr};
use rir::rtp::{RtpSession, RtpPkt, RtpHeader};
use sdp;

lazy_static! {
    static ref convos_by_name: Mutex<HashMap<String, Arc<Conference>>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
}

pub struct Conference {
    pub id: String,
    pub members: Mutex<HashMap<String, Arc<Mutex<Member>>>>,

    // SDP bound to the conference. The first member to arrive sets
    // sets the SDP which the other member will have to accept.
    sdp: Mutex<Option<SessionDescription>>
}

impl Conference {

    pub fn new(id: &str) -> Arc<Conference> {
        unsafe {
            if convos_by_name.lock().unwrap().contains_key(id) {
                return convos_by_name.lock().unwrap().get(id).unwrap().clone();
            }
        }

        debug!("Creating a new convo [{}]", id);

        let convo = Conference {
            id: id.to_string(),
            members: Mutex::new(HashMap::new()),
            sdp: Mutex::new(None),
        };

        unsafe {
            convos_by_name.lock().unwrap().insert(id.to_string(), Arc::new(convo));
            return convos_by_name.lock().unwrap().get(id).unwrap().clone();
        }
    }

    pub fn get(id: &str) -> Option<Arc<Conference>> {
        unsafe {
            if convos_by_name.lock().unwrap().contains_key(id) {
                return Some(convos_by_name.lock().unwrap().get(id).unwrap().clone());
            } else {
                return None;
            }
        }
    }

    /*fn change_ips(sdp: &mut SessionDescription, sock_addr: SocketAddr) {
        let mut origin = sdp.origin.clone().unwrap();
        origin.ip_address = sock_addr.ip();
        sdp.origin = Some(origin);

        for i in 0..sdp.media.len() {
            sdp.media[i].media.port = sock_addr.port();
        }
    }*/

    pub fn add_member(&self, member: Arc<Mutex<Member>>) -> Option<SessionDescription> {

        let mut mutex = self.sdp.lock().unwrap();
        let mut sdp_answer_to_ret;
        let var = match *mutex {
            // If there's still no SDP bound to this convo, this is
            // the one
            Some(ref convo) => { 
                debug!("Negotiating SDP with the conference");
                let mut sdp_answer = sdp::negotiate_with(Some(convo), &member.lock().unwrap().sdp);
                //member.lock().unwrap().init_audio();

                //Conference::change_ips(&mut sdp_answer, member.write().unwrap().rtp_session.as_ref().unwrap().conn.local_addr().unwrap());

                sdp_answer_to_ret = None;

                Some(sdp_answer)
            },
            None => {
                // TODO Even though this is the first SDP, it still
                //      needs to be negotiated with the platform
                debug!("Negotiating SDP with the platform");
                //let mut sdp_answer:SessionDescription = sdp::negotiate_with(None, &member.lock().unwrap().sdp);
                let mut member_lock = member.lock().unwrap();
                member_lock.negotiate_session(None);
                let sdp_answer = member_lock.get_session_answer();

                //member_lock.init_audio();
//                let rtp_stream = RtpSession::connect_to(UdpSocket::bind("192.168.2.186:6000").unwrap(), "0.0.0.0:0".parse().unwrap());

                //Conference::change_ips(&mut sdp_answer, member.read().unwrap().rtp_session.as_ref().unwrap().conn.local_addr().unwrap());

                sdp_answer_to_ret = Some(sdp_answer.clone());

                /**mutex = */Some(sdp_answer.clone())

                //self.sdp.lock().unwrap().clone()
            },
        };

        *mutex = sdp_answer_to_ret;

        self.members.lock().unwrap().insert(member.lock().unwrap().id.clone(), member.clone());

        var
    }

    pub fn get_member(&self, id: &str) -> Option<Arc<Mutex<Member>>>  {
        if self.members.lock().unwrap().contains_key(id) {
            return Some(self.members.lock().unwrap().get(id).unwrap().clone());
        } else {
            return None;
        }
    }

    pub fn process_engine(&self, member_local: Arc<Mutex<Member>>) {

        debug!("Processing engine...");

        let mutex = member_local.lock().unwrap();
        let mut rtp_pkt = mutex.read_audio();

        debug!("Writing packet...");

        mutex.write_audio(&rtp_pkt);
        /*
        for member in self.members.lock().unwrap().values() /*i in 0..self.members.len()*/ {
            let member_media = member.lock().unwrap().sdp.media[0].clone().media.port;
            debug!("Writing packet... {}", member_media);
            let member_local_media = member_local.lock().unwrap().sdp.media[0].clone().media.port;
            debug!("Writing packet... {}", member_local_media);
            if member_media != member_local_media {
                member.lock().unwrap().write_audio(&rtp_pkt);
            }
        }
        */
    }
}
