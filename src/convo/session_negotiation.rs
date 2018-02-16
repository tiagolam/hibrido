use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::boxed::Box;
use std::sync::{Arc, Mutex, RwLock};

use rir::rtp::{RtpSession, RirHandler};
use rir::handlers::{CallbackType};
use sdp::{SessionDescription, Attr, CandidateValue};
use convo::member::{Member};
use ice;
use sdp;

enum SessionState {
    CheckingOffer,
}

pub struct Session {
    offer_sdp: RwLock<SessionDescription>,
    base_sdp: RwLock<Option<SessionDescription>>,
    pub answer_sdp: RwLock<Option<SessionDescription>>,
    state: SessionState,
    ice: Arc<Mutex<ice::Agent>>,
    /* This is currently being used so that we can retrieve the stream_id
     * just by the SDP's ordered 'm' lines. iT was a quick hack, there must
     * be a better way.
     */
    sdp_to_ice: RwLock<Vec<String>>,
    pub media_sessions: Arc<RwLock<HashMap<String, RtpSession>>>,
    set_session: Option<Arc<Fn(&mut Member) + Send + Sync>>,
}

struct SessionRtp {
    stream_id: String,
    component_id: u16,
    ice: Arc<Mutex<ice::Agent>>,
    local_candidate: ice::Candidate,
}

impl RirHandler for SessionRtp {
    fn handle_event(&self, callback_type: CallbackType) {
        debug!("Received callback {:?} with component_id {}!", callback_type, self.component_id);

        match callback_type {
            CallbackType::USE_CANDIDATE(addr) => {
                self.ice.lock().unwrap().add_pair_candidate(&self.stream_id, &self.component_id, self.local_candidate.port, addr.port());
                // TODO(tlam): Move this inside ice.rs, so this check happens automatically when a
                // stream is completed
                self.ice.lock().unwrap().set_ice_complete();
            }
        }
    }
}

#[derive(Clone)]
struct SessionIce {
    media_sessions: Arc<RwLock<HashMap<String, RtpSession>>>,
}

// TODO(tlam): Get callbacks from ICE lib and eliminate / deallocate unused sessions.
impl ice::Handler for SessionIce {
    fn handle_callback(&mut self, stream_id: &str, peer: ice::Candidate) {
        debug!("Received ICE callback for stream_id {} and media_sessions {}", stream_id, self.media_sessions.read().unwrap().len());
        let media_lock = self.media_sessions.read().unwrap();
        let media_session = media_lock.get(stream_id);

        match media_session {
            Some(s) => {
                debug!("Set member's media session for stream {}", stream_id);
                if peer.component_id.unwrap() == 1 {
                    debug!("Set member's rtp peer to port {} for stream {}", peer.port, stream_id);
                    s.change_transport(SocketAddr::new(peer.conn, peer.port));
                }
                if peer.component_id.unwrap() == 2 {
                    debug!("Set member's rtcp peer to port {} for stream {}", peer.port, stream_id);
                    s.change_rtcp_transport(SocketAddr::new(peer.conn, peer.port));
                }
            },
            None => {
                info!("No media found for stream_id {}", stream_id);
            },
        }
    }
}

impl Session {
    // TODO(tlam): Do NOT assume ICE support
    pub fn new(offer_sdp: SessionDescription) -> Session {

        let media_sessions = Arc::new(RwLock::new(HashMap::new()));

        let session_ice = SessionIce {
            media_sessions: media_sessions.clone(),
        };

        let ice = ice::Agent::new(Box::new(session_ice));
        let session = Session {
            offer_sdp: RwLock::new(offer_sdp),
            base_sdp: RwLock::new(None),
            answer_sdp: RwLock::new(None),
            state: SessionState::CheckingOffer,
            ice: Arc::new(Mutex::new(ice)),
            sdp_to_ice: RwLock::new(Vec::new()),
            media_sessions: media_sessions,
            set_session: None,
        };

        session
    }

    pub fn ice_support() -> bool {
        // Check for ICE support within the offer
        true
    }

    pub fn init(&mut self, set_session: Arc<Fn(&mut Member) + Send + Sync>) {
        self.set_session = Some(set_session)
    }

    pub fn process_offer(&self) {
        // Create media stream and gather candidates for each stream
        for media in self.offer_sdp.read().unwrap().media.iter() {
            let mut ice = self.ice.lock().unwrap();
            let stream_id = ice.add_stream();

            ice.gather_candidates(&stream_id, &ice::RTP_COMPONENT_ID);
            ice.gather_candidates(&stream_id, &ice::RTCP_COMPONENT_ID);

            self.sdp_to_ice.write().unwrap().push(stream_id.clone());

            for attr in media.attrs.iter() {
                match *attr {
                    Attr::Candidate(ref c) => {
                        ice.add_offer_candidate(&stream_id, &c.ice_candidate.component_id.unwrap(), c.ice_candidate.clone());
                    },
                    _ => {},
                }
            }
        }
    }

    pub fn process_answer(&self) {
        // Add final candidates gathered for each stream

        let mut i = 0;
        // TODO(tlam): We are cloning here because there would be an immutable
        // reference to iter_mut vs the mutable reference to call
        // init_media_session
        for media in self.answer_sdp.write().unwrap().as_mut().unwrap().media.iter_mut() {
            let ref stream_id = self.sdp_to_ice.read().unwrap()[i];

            //let tmp_candidates = Vec::new();
            let ice = self.ice.lock().unwrap();
            let mut candidates = ice.get_stream_candidates(stream_id, &ice::RTP_COMPONENT_ID).unwrap().clone();/*_or(&mut tmp_candidates)*/;
            let candidates_rtcp = ice.get_stream_candidates(stream_id, &ice::RTCP_COMPONENT_ID).unwrap();
            for candidate_rtcp in candidates_rtcp.iter() {
                candidates.push(candidate_rtcp.clone());
            }

            for candidate in candidates.iter() {
                debug!("Adding candidate {}:{}", candidate.conn.to_string(), candidate.port);
                // Add candidate to the final SDP answer
                media.attrs.push(Attr::Candidate(CandidateValue {
                    ice_candidate: candidate.clone()
                }));
            }

            i += 1;
        }

        i = 0;
        // Start new media session on the candidate
        for _ in self.answer_sdp.read().unwrap().as_ref().unwrap().media.iter() {
            let ref stream_id = self.sdp_to_ice.read().unwrap()[i];

            let ice = self.ice.lock().unwrap();
            let mut rtp_candidates = ice.get_stream_candidates(stream_id, &ice::RTP_COMPONENT_ID).unwrap().clone();
            let rtcp_candidates = ice.get_stream_candidates(stream_id, &ice::RTCP_COMPONENT_ID).unwrap();
            if rtp_candidates.len() != rtcp_candidates.len() {
                warn!("Different number of candidates for RTP and RTCP {}!={}", rtp_candidates.len(), rtcp_candidates.len());
            }

            for it in rtp_candidates.iter().zip(rtcp_candidates.iter()) {
                let (rtp_candidate, rtcp_candidate) = it;
                // Start new media session on the candidates
                debug!("Init candidate stream {}:{}", rtp_candidate.conn.to_string(), rtp_candidate.port);
                let media_session = self.init_media_session(stream_id.to_string(), rtp_candidate, rtcp_candidate);
                self.media_sessions.write().unwrap().insert(stream_id.to_string(), media_session);
            }

            i += 1;
        }
    }

    pub fn negotiate_with_base_sdp(&self, base_sdp: Option<SessionDescription>) {
        // Negotiate base SDP with SDP offer
        // The SDP answer will come out of this, and will need to be put
        // through process_answer
        /*if !base_sdp.is_some() {
            self.answer_sdp = Some(self.offer_sdp.clone());

            return;
        }*/

        let mut bsdp_lock = self.base_sdp.write().unwrap();
        *bsdp_lock = base_sdp;

        let sdp_answer = sdp::negotiate_with(bsdp_lock.as_ref(), &self.offer_sdp.read().unwrap());

        let mut asdp_lock = self.answer_sdp.write().unwrap();
        *asdp_lock = Some(sdp_answer);
    }

    pub fn init_media_session(&self, stream_id: String, rtp_candidate: &ice::Candidate, rtcp_candidate: &ice::Candidate) -> RtpSession {

        let component_id = rtp_candidate.component_id.unwrap();
        let rtp_handler = SessionRtp {
            stream_id: stream_id.clone(),
            component_id: component_id,
            ice: self.ice.clone(),
            local_candidate: rtp_candidate.clone(),
        };
        let rtp_cb: Box<RirHandler + Send> = Box::new(rtp_handler);
        let rtp_conn = UdpSocket::bind(SocketAddr::new(rtp_candidate.conn, rtp_candidate.port));

        let component_id = rtcp_candidate.component_id.unwrap();
        let rtcp_handler = SessionRtp {
            stream_id: stream_id.clone(),
            component_id: component_id,
            ice: self.ice.clone(),
            local_candidate: rtcp_candidate.clone(),
        };
        let rtcp_cb: Box<RirHandler + Send> = Box::new(rtcp_handler);
        let rtcp_conn = UdpSocket::bind(SocketAddr::new(rtcp_candidate.conn, rtcp_candidate.port));

        let rtp_session = new_rtp_session(rtp_conn.unwrap(), rtcp_conn.unwrap(), self.offer_sdp.read().unwrap().clone(), rtp_cb, rtcp_cb);

        rtp_session
    }
}

pub fn new_rtp_session(rtp_conn: UdpSocket, rtcp_conn: UdpSocket, sdp: SessionDescription, rtp_cb: Box<RirHandler + Send>, rtcp_cb: Box<RirHandler + Send>) -> RtpSession {

    let ip_addr = rtp_conn.local_addr().unwrap().ip();
    let port = sdp.media[0].media.port;

    debug!("Connecting to remote endpoint {}:{}", ip_addr, port);

    let rtp_stream = RtpSession::connect_to(rtp_conn, rtcp_conn, SocketAddr::new(ip_addr, port), rtp_cb, rtcp_cb);

    rtp_stream
}

