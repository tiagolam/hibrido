use std::collections::HashMap;

use sdp::{SessionDescription, Attr, CandidateValue};
use ice;
use sdp;

enum SessionState {
    CheckingOffer,
}

pub struct Session {
    offer_sdp: SessionDescription,
    base_sdp: Option<SessionDescription>,
    answer_sdp: Option<SessionDescription>,
    state: SessionState,
    ice: ice::Agent,
    sdp_to_ice: Vec<String>,
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
        for media in self.offer_sdp.media.iter_mut() {
            let ref stream_id = self.sdp_to_ice[i];

            let candidates = self.ice.get_stream_candidates(stream_id, &ice::RTP_COMPONENT_ID);

            for candidate in candidates.iter() {
                debug!("Adding candidate {}", candidate.conn.to_string());
                media.attrs.push(Attr::Candidate(CandidateValue {
                    ice_candidate: candidate.clone()
                }));
            }

            i += 1;
        }
    }

    pub fn negotiate_with_base_sdp(&mut self, base_sdp: SessionDescription) {
        // Negotiate base SDP with SDP offer
        // The SDP answer will come out of this, and will need to be put
        // through process_answer
        self.base_sdp = Some(base_sdp);

        let mut sdp_answer = sdp::negotiate_with(self.base_sdp.as_ref(), &self.offer_sdp);
        self.answer_sdp = Some(sdp_answer);
    }
}

