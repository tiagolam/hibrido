use sdp;
use ice;

enum SessionState {
    CheckingOffer,
}

struct Session {
    offer_sdp: SessionDescription,
    base_sdp: SessionDescription,
    answer_sdp: SessionDescription,
    state: SessionState,
    ice: ice::Ice,
}

pub impl Session {

    pub new(offer_sdp: SessionDescription) -> Session {

        let mut ice;
        if ice_support() {
            ice = ice::Ice::new();
            ice.start_agent();
        }

        ice.gather_candidates

        Session {
            offer_sdp: offer_sdp,
            // Start ice agent
            ice: ice::Ice::new(),
        }
    }

    pub ice_support() -> bool {
        // Check for ICE support within the offer
    }

    pub gather_candidates(&self) {
        // ICE Lite gathring process only gathers HOST candidates

        // For each media session gather candidates accordingly

        self.ice.gather_candidates(ice::CandidateTypes::Host);
    }

    pub verify_answer() {

        self.i
    }

    pub negotiate_with_base_sdp(base_sdp: SessionDescription) -> SdpDescription{

    }
}

