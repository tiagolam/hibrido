use lazy_static;
use std::collections::HashMap;
use std::boxed::Box;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use convo::member::Member;
use sdp::{SessionDescription};
use std::net::{UdpSocket, SocketAddr};
use rir::rtp::{RtpSession, RtpPkt, RtpHeader};
use std::fs::File;
use std::io::Write;
use std::io::Read;
use sdp;

lazy_static! {
    static ref convos_by_name: Mutex<HashMap<String, Arc<Conference>>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
}

pub struct Conference {
    pub id: String,
    pub members: Arc<Mutex<HashMap<String, Arc<Member>>>>,
    // SDP bound to the conference. The first member to arrive sets
    // sets the SDP which the other member will have to accept.
    sdp: Mutex<Option<SessionDescription>>,
}

impl Conference {

    pub fn new(id: &str) -> Arc<Conference> {
        if convos_by_name.lock().unwrap().contains_key(id) {
            return convos_by_name.lock().unwrap().get(id).unwrap().clone();
        }

        debug!("Creating a new convo [{}]", id);

        let convo = Conference {
            id: id.to_string(),
            members: Arc::new(Mutex::new(HashMap::new())),
            sdp: Mutex::new(None),
        };

        convos_by_name.lock().unwrap().insert(id.to_string(), Arc::new(convo));
        return convos_by_name.lock().unwrap().get(id).unwrap().clone();
    }

    pub fn get(id: &str) -> Option<Arc<Conference>> {
        if convos_by_name.lock().unwrap().contains_key(id) {
            return Some(convos_by_name.lock().unwrap().get(id).unwrap().clone());
        } else {
            return None;
        }
    }

    pub fn add_member(&self, member: Member) -> Option<SessionDescription> {
        let mut mutex = self.sdp.lock().unwrap();

        member.init_session();

        let mut sdp_answer_to_ret;
        let var = match *mutex {
            // If there's still no SDP bound to this convo, this is
            // the one
            Some(ref convo) => { 
                debug!("Negotiating SDP with the conference");
                member.negotiate_session(Some(convo.clone()));
                let sdp_answer = member.get_session_answer();

                sdp_answer_to_ret = Some(sdp_answer.clone());

                Some(sdp_answer.clone())
            },
            None => {
                // TODO Even though this is the first SDP, it still
                //      needs to be negotiated with the platform
                debug!("Negotiating SDP with the platform");
                member.negotiate_session(None);
                let sdp_answer = member.get_session_answer();

                sdp_answer_to_ret = Some(sdp_answer.clone());

                /* Start engine, this is the first bound SDP */
                self.process_engine();

                Some(sdp_answer.clone())
            },
        };

        *mutex = sdp_answer_to_ret;

        self.members.lock().unwrap().insert(member.id.clone(), Arc::new(member));

        var
    }

    pub fn get_member(&self, id: &str) -> Option<Arc<Member>>  {
        if self.members.lock().unwrap().contains_key(id) {
            return Some(self.members.lock().unwrap().get(id).unwrap().clone());
        } else {
            return None;
        }
    }

    fn process_engine(&self) {
        debug!("Processing engine...");
        let members = self.members.clone();
        //let mut file = File::open("/home/lam/Downloads/a2002011001-e02-16kHz.wav").unwrap();
        let mut file = File::create("/tmp/a2002011001-e02-16kHz_dup.wav").unwrap();
        let mut buffer: [u8; 640] = [0; 640];
        thread::spawn(move || {
            loop {
                let members = members.lock().unwrap();

                for (_, member) in members.iter() {
                    thread::sleep(time::Duration::from_millis(1));
                    debug!("Reading from member buffer...");

                    let payload = member.get_read_payload();

                    if !payload.is_some() {
                        continue
                    }

                    debug!("Writing audio packet...");

                    let payload = payload.unwrap();

                    member.set_write_payload(payload);
                    /*let read = file.read(&mut buffer);
                    match read {
                        Ok(x) => {
                            if x == 0 {
                                continue;
                            }

                            member.set_write_payload(buffer);
                        },
                        Err(e) => {
                            debug!("Error reading file... {}", e);
                        },
                    }*/

                    debug!("After writing to file...");

                    file.write(&payload);
                }
            }
        });
    }
}
