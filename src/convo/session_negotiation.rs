use std::collections::HashMap;
use std::net::{UdpSocket, IpAddr, SocketAddr};
use std::boxed::Box;

use rir::rtp::{RtpSession};
use rir::handlers::{CallbackType};
use sdp::{SessionDescription, Attr, CandidateValue};
use ice;
use sdp;

enum SessionState {
    CheckingOffer,
}

pub struct Session {
    offer_sdp: SessionDescription,
    base_sdp: Option<SessionDescription>,
    pub answer_sdp: Option<SessionDescription>,
    state: SessionState,
    ice: ice::Agent,
    sdp_to_ice: Vec<String>,
    media_sessions: HashMap<String, RtpSession>,
}

impl Session {

    // TODO(tlam): Do NOT assume ICE support
    pub fn new(offer_sdp: SessionDescription) -> Session {
        let ice = ice::Agent::new();
        ice.start();

        Session {
            offer_sdp: offer_sdp,
            base_sdp: None,
            answer_sdp: None,
            state: SessionState::CheckingOffer,
            ice: ice,
            sdp_to_ice: Vec::new(),
            media_sessions: HashMap::new(),
        }
    }

    pub fn ice_support() -> bool {
        // Check for ICE support within the offer
        true
    }

    pub fn process_offer(&mut self) {
        // Create media stream and gather candidates for each stream
        for media in self.offer_sdp.media.iter() {
            let stream_id = self.ice.add_stream();

            self.ice.gather_candidates(&stream_id, &ice::RTP_COMPONENT_ID);
            //self.gather_candidates(stream_id, ice::RTCP_COMPONENT_ID);

            self.sdp_to_ice.push(stream_id);
        }
    }

    pub fn process_answer(&mut self) {
        // Add final candidates gathered for each stream

        let mut i = 0;
        // TODO(tlam): We are cloning here because there would be an immutable
        // reference to iter_mut vs the mutable reference to call
        // init_media_session
        for media in self.answer_sdp.as_mut().unwrap().media.iter_mut() {
            let ref stream_id = self.sdp_to_ice[i];

            let tmp_candidates = Vec::new();
            let candidates = self.ice.get_stream_candidates(stream_id, &ice::RTP_COMPONENT_ID).unwrap_or(&tmp_candidates);

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
        for media in self.answer_sdp.as_ref().unwrap().media.iter() {
            let ref stream_id = self.sdp_to_ice[i];

            let tmp_candidates = Vec::new();
            let candidates = self.ice.get_stream_candidates(stream_id, &ice::RTP_COMPONENT_ID).unwrap_or(&tmp_candidates);

            for candidate in candidates.iter() {
                // Start new media session on the candidate
                debug!("Init candidate stream {}:{}", candidate.conn.to_string(), candidate.port);
                let media_session = self.init_media_session(candidate.conn, candidate.port);
                self.media_sessions.insert(stream_id.to_string(), media_session);
            }

            i += 1;
        }
    }

    pub fn negotiate_with_base_sdp(&mut self, base_sdp: Option<SessionDescription>) {
        // Negotiate base SDP with SDP offer
        // The SDP answer will come out of this, and will need to be put
        // through process_answer
        /*if !base_sdp.is_some() {
            self.answer_sdp = Some(self.offer_sdp.clone());

            return;
        }*/

        self.base_sdp = base_sdp;

        let mut sdp_answer = sdp::negotiate_with(self.base_sdp.as_ref(), &self.offer_sdp);
        self.answer_sdp = Some(sdp_answer);
    }

    pub fn init_media_session(&self, conn: IpAddr, port: u16) -> RtpSession {
        let bind_socket = SocketAddr::new(conn, port);
        let conn = UdpSocket::bind(bind_socket);

        let rtp_session = new_rtp_session(conn.unwrap(), self.offer_sdp.clone(), Box::new(use_candidate_callback));

        rtp_session
    }
}

pub fn use_candidate_callback(callback_type: CallbackType) {
    debug!("Received callback {:?}!", callback_type);
}

pub fn new_rtp_session(conn: UdpSocket, sdp: SessionDescription, callback: Box<Fn(CallbackType) + Send>) -> RtpSession {

    let ip_addr = sdp.origin.unwrap().ip_address;
    let port = sdp.media[0].media.port;

    debug!("Connecting to endpoint {}:{}", ip_addr, port);

    let rtp_stream = RtpSession::connect_to(conn, SocketAddr::new(ip_addr, port), callback);

    rtp_stream
}

