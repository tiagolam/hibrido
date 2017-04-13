use std::collections::HashMap

use sdp;
use ice;

enum SessionState {
    CheckingOffer,
}

struct Session {
    offer_sdp: SessionDescription,
    base_sdp: Option<SessionDescription>,
    answer_sdp: Option<SessionDescription>,
    state: SessionState,
    ice: ice::Agent,
    sdp_to_ice: Vec<String>,
}

pub impl Session {

    // TODO(tlam): Do NOT assume ICE support
    pub fn new(offer_sdp: SessionDescription) -> Session {

        let ice = ice::Agent::new();
        ice.start();

        Session {
            offer_sdp: offer_sdp,
            base_sdp: None,
            answer_sdp: None,
            state: SessionState::CheckingOffer
            ice: ice,
            sdp_to_ice: Vec::new(),
        }
    }

    pub ice_support() -> bool {
        // Check for ICE support within the offer
    }

    pub fn process_offer(&self) {
        // Create media stream and gather candidates for each stream
        for media in self.offer_sdp.media.iter() {
            let stream_id = self.ice.add_stream();

            self.gather_candidates(stream_id, ice::RTP_COMPONENT_ID);
            //self.gather_candidates(stream_id, ice::RTCP_COMPONENT_ID);

            self.sdp_to_ice.push(stream_id);
        }
    }

    pub fn process_answer(&self) {
        // Add final candidates gathered for each stream

        let mut i = 0;
        for media in self.offer_sdp.media.iter() {
            let stream_id = self.sdp_to_ice[i];

            let candidates = self.ice.get_stream_candidates(stream_id, ice::RTP_COMPONENT_ID);

            for candidate in candidates.iter() {
                media.attrs.push(Attr::Candidate(CandidateValue {
                    ice_candidate: candidate
                }));
            }

            i += 1;
        }
    }

    pub fn negotiate_with_base_sdp(&self, base_sdp: SessionDescription) {
        // Negotiate base SDP with SDP offer
        // The SDP answer will come out of this, and will need to be put
        // through process_answer
        self.base_sdp = base_sdp.clone();

        let mut sdp_answer = sdp::negotiate_with(base_sdp, self.offer_sdp);
        self.answer_sdp = sdp_answer;
    }
}

