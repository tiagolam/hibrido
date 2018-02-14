use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use convo::member::Member;
use sdp::{SessionDescription};

lazy_static! {
    static ref CONVOS_BY_NAME: Mutex<HashMap<String, Arc<Conference>>> = {
        let m = HashMap::new();
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
        if CONVOS_BY_NAME.lock().unwrap().contains_key(id) {
            return CONVOS_BY_NAME.lock().unwrap().get(id).unwrap().clone();
        }

        debug!("Creating a new convo [{}]", id);

        let convo = Conference {
            id: id.to_string(),
            members: Arc::new(Mutex::new(HashMap::new())),
            sdp: Mutex::new(None),
        };

        CONVOS_BY_NAME.lock().unwrap().insert(id.to_string(), Arc::new(convo));
        return CONVOS_BY_NAME.lock().unwrap().get(id).unwrap().clone();
    }

    pub fn get(id: &str) -> Option<Arc<Conference>> {
        if CONVOS_BY_NAME.lock().unwrap().contains_key(id) {
            return Some(CONVOS_BY_NAME.lock().unwrap().get(id).unwrap().clone());
        } else {
            return None;
        }
    }

    pub fn add_member(&self, member: Member) -> Option<SessionDescription> {
        let mut mutex = self.sdp.lock().unwrap();

        member.init_session();

        let sdp_answer_to_ret;
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
        thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_millis(1));
                let members = members.lock().unwrap();

                /* Read from members */
                for (_, member) in members.iter() {
                    let mut tmp_payload: [u8; 3840] = [0; 3840];
                    let mut write = false;
                    for (_, member_r) in members.iter() {
                        if member.id != member_r.id {
                            debug!("Reading from member {}...", member_r.id);

                            let payload = member.get_read_payload();
                            if !payload.is_some() {
                                continue
                            }

                            write = true;
                            tmp_payload  = sum_payload(tmp_payload, payload.unwrap());
                        }
                    }

                    if write {
                        debug!("Writing audio packet to member {}...", member.id);

                        member.set_write_payload(tmp_payload);
                    }
                }

            }
        });
    }
}

fn sum_payload(payload1: [u8; 3840], payload2: [u8; 3840]) -> [u8; 3840] {
    let mut result: [u8; 3840] = [0; 3840];

    for i in 0..payload1.len() {
        result[i] = payload1[i] + payload2[i];
    }

    result
}
