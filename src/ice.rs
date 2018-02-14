extern crate ifaces;
extern crate rustun;
extern crate fibers;
extern crate uuid;
extern crate timer;
extern crate time;

use std::str::FromStr;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::collections::HashMap;
use self::uuid::Uuid;
use std::sync::{mpsc};

use self::timer::Timer;
use self::time::Duration;

pub const RTP_COMPONENT_ID: u16 = 1;
pub const RTCP_COMPONENT_ID: u16 = 2;

#[derive(Clone, Debug, PartialEq)]
pub enum Proto {
    Udp,
    Tcp,
}

impl ToString for Proto {
    fn to_string(&self) -> String {
        match *self {
            Proto::Udp => return "udp".to_string(),
            Proto::Tcp => return "tcp".to_string(),
        }
    }
}

impl FromStr for Proto {
    type Err = ();

    fn from_str(s: &str) -> Result<Proto, ()> {
        match s {
            "udp" => Ok(Proto::Udp),
            "UDP" => Ok(Proto::Udp),
            "tcp" => Ok(Proto::Tcp),
            "TCP" => Ok(Proto::Tcp),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CandidateType {
    Host,
    Srflx,
    Prflx,
    Relay,
}

impl ToString for CandidateType {
    fn to_string(&self) -> String {
        match *self {
            CandidateType::Host => return "host".to_string(),
            CandidateType::Srflx => return "srflx".to_string(),
            CandidateType::Prflx => return "prflx".to_string(),
            CandidateType::Relay => return "relay".to_string(),
        }
    }
}

impl FromStr for CandidateType {
    type Err = ();

    fn from_str(s: &str) -> Result<CandidateType, ()> {
        match s {
            "host" => Ok(CandidateType::Host),
            "srflx" => Ok(CandidateType::Srflx),
            "prflx" => Ok(CandidateType::Prflx),
            "relay" => Ok(CandidateType::Relay),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub conn: IpAddr,
    pub port: u16,
    pub proto: Proto,
    pub foundation: String,
    pub component_id: Option<u16>,
    pub priority: u32,
    pub candidate_type: CandidateType,
    pub rel_addr: Option<IpAddr>,
    pub rel_port: Option<u16>,
}

pub struct PairCandidate {
    // TODO(tlam): Use references and lifetimes here
    local_candidate: Candidate,
    peer_candidate: Candidate,
}

#[derive(PartialEq)]
enum StreamState {
    Running,
    Completed,
}

struct Stream {
    id: String,
    state: StreamState,
    check_list: HashMap<u16, Vec<PairCandidate>>,
    valid_list: HashMap<u16, Vec<PairCandidate>>,
    offer_candidates: HashMap<u16, Vec<Candidate>>,
    local_candidates: HashMap<u16, Vec<Candidate>>,
}

enum IceState {
    Running,
    Completed,
}

pub struct Agent {
    state: IceState,
    streams: HashMap<String, Stream>,
    handler: Option<Box<Handler + Send>>,
}

static mut START_PORT: u16 = 6000;

/// Get IPv4 addresses only.
fn get_ipv4_address() -> Option<SocketAddr> {

    let mut ipv4_addr = None;
    for iface in
        ifaces::Interface::get_all().unwrap()
                                    .into_iter() {
        debug!("{}\t{:?}\t{:?}", iface.name, iface.kind, iface.addr);

        // Avoid localhost IPs
        if iface.name != "lo" {
            match iface.kind {
                ifaces::Kind::Ipv4 => {
                    ipv4_addr = iface.addr;
                    debug!("Chosen {:?}", iface.addr);

                    // Return first IPv4 address found
                    break;
                },
                _ => {},
            };
        }
    }

    ipv4_addr
}

pub trait Handler {
    fn handle_callback(&mut self, &str, Candidate);
}

impl Agent {
    pub fn new(handler: Box<Handler + Send>) -> Agent {
        Agent {
            state: IceState::Running,
            streams: HashMap::new(),
            // TODO(tlam): Miss the Option wrapper so we can use this without
            // having to place checks all around the callbacks code
            handler: Some(handler),
        }
    }

    /// Start agent and initiate the regular functions
    pub fn start(&mut self, handler: Box<Handler + Send>) {
        let (tx, rx) = mpsc::channel();
        let timer = Timer::new();

        self.handler = Some(handler);

//        loop {
            timer.schedule_with_delay(Duration::milliseconds(10),
                move || {
                    let err = tx.send(1);
                    match err {
                        Err(x) => {
                            error!("Problem scheduling timer: {}", x);
                        },
                        _ => {},
                    }
                }
            );

            let err = rx.recv();
            match err {
                Err(x) => {
                    error!("Problem scheduling timer: {}", x);
                    return;
                },
                _ => {},
            }

            let mut found = false;
            for (_, stream) in self.streams.iter() {
                if stream.state != StreamState::Completed {
                    found = true;
                }
            }
            if !found {
                self.state = IceState::Completed;
            }

//        }
    }

    pub fn set_ice_complete(&mut self) {
        for (_, stream) in self.streams.iter() {
            if stream.state != StreamState::Completed {
                return;
            }
        }

        self.state = IceState::Completed;
    }

    /// Add new stream to the current agent, of the component provided
    pub fn add_stream(&mut self) -> String {
        // Add stream to agent
        let stream_id: &str = &Uuid::new_v4().to_string();
        let stream = Stream {
            id: stream_id.to_string(),
            state: StreamState::Running,
            check_list: HashMap::new(),
            valid_list: HashMap::new(),
            offer_candidates: HashMap::new(),
            local_candidates: HashMap::new(),
        };

        self.streams.insert(stream_id.to_string(), stream);

        stream_id.to_string()
    }

    pub fn add_offer_candidate(&mut self, stream_id: &str, component_id: &u16, candidate: Candidate) {
        let stream = match self.streams.get_mut(stream_id) {
            Some(stream) => { stream },
            None => { return },
        };

        let candidates: &mut Vec<Candidate> = stream.offer_candidates.entry(*component_id).or_insert(Vec::new());
        candidates.push(candidate);
    }

    pub fn get_stream_candidates(&self, stream_id: &str, component_id: &u16) -> Option<&Vec<Candidate>> {
        let stream = match self.streams.get(stream_id) {
            Some(stream) => { stream },
            None => { return None },
        };

        stream.local_candidates.get(component_id)
    }

    /// Gather candidates for a particular stream
    pub fn gather_candidates(&mut self, stream_id: &str, component_id: &u16) {
        let stream = self.streams.get_mut(stream_id).unwrap();

        let ipv4_addr = get_ipv4_address();
        if !ipv4_addr.is_some() {
            return
        }

        let candidates: &mut Vec<Candidate> = stream.local_candidates.entry(*component_id).or_insert(Vec::new());

        let mut ipv4_addr = ipv4_addr.unwrap();
        unsafe {
            ipv4_addr.set_port(START_PORT);
            START_PORT += 1;
        }
        //let conn = UdpSocket::bind(ipv4_addr).unwrap();

        let port = ipv4_addr.port();

        //let skt = SocketAddr::new(ipv4_addr, port);

        //setup_stun_server(conn);
        //let rtp_stream = RtpSession::connect_to(conn, "0.0.0.0:0".parse().unwrap())

        // Get new candidate
        let mut candidate = Candidate {
            conn: ipv4_addr.ip(),
            port: port,
            proto: Proto::Udp,
            foundation: "deadbeef".to_string(),
            component_id: Some(*component_id),
            priority: 0,
            candidate_type: CandidateType::Host,
            rel_addr: None,
            rel_port: None,
        };
        Agent::set_priority_candidate(&mut candidate, *component_id);

        candidates.push(candidate);
    }

    pub fn add_pair_candidate(&mut self, stream_id: &str, component_id: &u16, local_port: u16, remote_port: u16) {
        let stream = match self.streams.get_mut(stream_id) {
            Some(stream) => { stream },
            None => { return },
        };

        info!("New offered candidate for stream_id {}", stream_id);

        if stream.state == StreamState::Completed {
            debug!("Stream_id {} is complete", stream_id);
            return;
        }

        {
            /* Find remote_port amongst the offer candidates */
            let offer_candidates: &mut Vec<Candidate> = stream.offer_candidates.entry(*component_id).or_insert(Vec::new());
            let mut peer_candidate: Option<Candidate> = None;
            for candidate in offer_candidates.iter() {
                if candidate.port == remote_port {
                    peer_candidate = Some(candidate.clone());
                    break;
                }
            }

            /* Find local_port amongst the local candidates */
            let local_candidates: &mut Vec<Candidate> = stream.local_candidates.entry(*component_id).or_insert(Vec::new());
            let mut local_candidate: Option<Candidate> = None;
            for candidate in local_candidates.iter() {
                if candidate.port == local_port {
                    local_candidate = Some(candidate.clone());
                    break;
                }
            }

            if peer_candidate.is_none() || local_candidate.is_none() {
                debug!("No pair inserted for local port {} and remote_port {}!", local_port, remote_port);
                return;
            }

            let pairs: &mut Vec<PairCandidate> = stream.valid_list.entry(*component_id).or_insert(Vec::new());
            pairs.push(PairCandidate {
                local_candidate: local_candidate.unwrap(),
                peer_candidate: peer_candidate.unwrap(),
            });
        }

        /* Check if stream has a pair for all components, in which case it is considered completed */
        if is_stream_complete(stream, &[RTP_COMPONENT_ID, RTCP_COMPONENT_ID]) {
            stream.state = StreamState::Completed;
            /* Choose which candidate pair to use and send it */
            /* XXX(tlam): As of now we're only sending the first */
            for (_, valid_list) in stream.valid_list.iter() {
                for v in valid_list.iter() {
                    self.handler.as_mut().unwrap().handle_callback(stream_id, v.peer_candidate.clone());
                    break;
                }
            }
        }
    }

    fn pair_candidates(&mut self, stream_id: &str, component_id: &u16) {
        // Compare remote candidates to local candidates as rfc#5245:
        // - They have same component;
        // - They utilize same transport protocol;
        // - Same IP family (IPv4 and IPv6).
        let stream = match self.streams.get_mut(stream_id) {
            Some(stream) => { stream },
            None => { return },
        };

        for candidate in stream.local_candidates.get(component_id).unwrap().iter() {
            for peer_candidate in stream.offer_candidates.get(component_id).unwrap().iter() {
                if candidate.proto == peer_candidate.proto {
                    if is_ipv4(&candidate.conn) && is_ipv4(&peer_candidate.conn) {
                        debug!("Found pair candidate! {:?}:{:?}", candidate.conn, peer_candidate.conn);
                        let pairs: &mut Vec<PairCandidate> = stream.valid_list.entry(*component_id).or_insert(Vec::new());
                        pairs.push(PairCandidate {
                            local_candidate: candidate.clone(),
                            peer_candidate: peer_candidate.clone(),
                        });
                    }
                }
            }
        }

        if trigger_state_change(stream, component_id) {
            // If there was a state change, trigger callback
            match self.handler {
                Some(ref mut _h) => {
                    //h.handle_callback(stream_id);
                },
                None => {
                    info!("Undelivered event. No callback set!");
                },
            }
        }
    }

    fn set_priority_candidate(candidate: &mut Candidate, component_id: u16) {
        let priority = ((2^24 as u32)*(126 as u32) +
                        (2^8 as u32)*(65535 as u32) + // 65535 from #rfc5245
                        (2^0 as u32)*((256 - component_id) as u32)) as u32;

        candidate.priority = priority;
    }
}

fn is_stream_complete(stream: &Stream, components: &[u16]) -> bool {
    for i in components.iter() {
        let valid_list = stream.valid_list.get(i);
        if valid_list.unwrap_or(&vec![]).len() < 1 {
            return false;
        }
    }

    return true;
}

fn trigger_state_change(stream: &mut Stream, _component_id: &u16) -> bool {
    // Check if valid list has pairs for all components
    // TODO(tlam): What if there is more than one candidate per component?
    // (which can happen in dual IPv4 and IPv6 stacks)
    if stream.local_candidates.len() == stream.valid_list.len() {
        stream.state = StreamState::Completed;

        return true
    }

    false
}

fn is_ipv4(conn: &IpAddr) -> bool {
    match *conn {
        IpAddr::V4(_) => {
            debug!("Ipv4 address found");

            return true
        },
        IpAddr::V6(_) => {
            debug!("Ipv6 address found");

            return false
        },
    }
}

