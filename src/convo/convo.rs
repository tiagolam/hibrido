use lazy_static;
use std::collections::HashMap;
use std::boxed::Box;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
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
    pub members: Mutex<Vec<Arc<RwLock<Member>>>>,

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
            members: Mutex::new(vec![]),
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

    fn change_ips(sdp: &mut SessionDescription, sock_addr: SocketAddr) {
        let mut media = sdp.media.clone().unwrap();
        let mut origin = sdp.origin.clone().unwrap();
        media.port = sock_addr.port();
        origin.ip_address = sock_addr.ip();
        sdp.origin = Some(origin);
        sdp.media = Some(media);
    }

    pub fn add_member(&self, member: Arc<RwLock<Member>>) -> Option<SessionDescription> {

        self.members.lock().unwrap().push(member.clone());

        let mut mutex = self.sdp.lock().unwrap();
        let mut sdp_answer_to_ret;
        let var = match (*mutex) {
            // If there's still no SDP bound to this convo, this is
            // the one
            Some(ref convo) => { 
                debug!("Negotiating SDPs");
                let mut sdp_answer = sdp::negotiate_with(Some(convo), &member.read().unwrap().sdp);

                member.write().unwrap().init_audio();

                Conference::change_ips(&mut sdp_answer, member.write().unwrap().rtp_session.as_ref().unwrap().conn.local_addr().unwrap());

                sdp_answer_to_ret = None;

                Some(sdp_answer)
            },
            None => {
                // TODO Even though this is the first SDP, it still
                //      needs to be negotiated with the platform
                debug!("Bonding incoming SDP");
                let mut sdp_answer:SessionDescription = sdp::negotiate_with(None, &member.read().unwrap().sdp);

                member.write().unwrap().init_audio();

                Conference::change_ips(&mut sdp_answer, member.read().unwrap().rtp_session.as_ref().unwrap().conn.local_addr().unwrap());

                sdp_answer_to_ret = Some(sdp_answer.clone());

                /**mutex = */Some(sdp_answer)
                
                //self.sdp.lock().unwrap().clone()
            },
        };

        *mutex = sdp_answer_to_ret; 

        var

        // XXX HOW TO:
        // Use ICE for the establishing the RTP session?
        // How to deal with TURN and STUN?

        // The SDP contains the candidates alreadt, so those should
        // be used in order to create the rtp sessions?
    }

    pub fn process_engine(&self, member_local: Arc<RwLock<Member>>) {

        let mut rtp_pkt = member_local.read().unwrap().read_audio();

        debug!("Writing packet...");

        for member in self.members.lock().unwrap().iter() /*i in 0..self.members.len()*/ {
            if member.read().unwrap().sdp.media.clone().unwrap().port != member_local.read().unwrap().sdp.media.clone().unwrap().port {
                member.read().unwrap().write_audio(&rtp_pkt);
            }
        }
    }
}

pub fn rm_member(member: Member) {
}

