extern crate ifaces;
extern crate rustun;
extern crate fibers;
extern crate uuid;

use std::str::FromStr;
use std::net::IpAddr;
use std::net::{UdpSocket, SocketAddr, SocketAddrV4};
use std::thread;
use std::collections::HashMap;
use self::uuid::Uuid;

use self::fibers::{Executor, InPlaceExecutor, Spawn};
use self::rustun::server::UdpServer;
use self::rustun::rfc5389::handlers::BindingHandler;

use rir::rtp::{RtpSession, RtpPkt, RtpHeader};

const RTP_COMPONENT_ID: u16 = 1;
const RTCP_COMPONENT_ID: u16 = 2;

#[derive(Clone, Debug)]
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

enum StreamState {
    Running,
}

struct Stream {
    id: String,
    state: StreamState,
    remote_candidates: HashMap<u16, Vec<Candidate>>,
    local_candidates: HashMap<u16, Vec<Candidate>>,
}

enum IceState {
    Running,
    Completed,
}

pub struct Agent {
    state: IceState,
    streams: HashMap<String, Stream>,
}

static mut START_PORT: u16 = 6000;

/// Setup STUN server on the port
fn setup_stun_server(conn: UdpSocket) {
    let mut executor = InPlaceExecutor::new().unwrap();

    let spawner = executor.handle();
    let monitor = executor.spawn_monitor(UdpServer::new(conn)
                          .start(spawner.boxed(), BindingHandler::new("T0teqPLNQQOf+5W+ls+P2p16".to_string())));
    let result = executor.run_fiber(monitor).unwrap();

    thread::spawn(move || {
        let result = executor.run_fiber(monitor).unwrap();
    });
}

/// Get IPv4 addresses only.
fn get_ipv4_address() -> Option<SocketAddr> {

    let ipv4_addr = None;

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

impl Agent {
    pub fn new() -> Agent {
        Agent {
            state: IceState::Running,
            streams: HashMap::new(),
        }
    }

    /// Start agent and initiate the regular functions
    pub fn start() {
    }

    /// Add new stream to the current agent, of the component provided
    pub fn add_stream(&mut self) -> String {
        // Add stream to agent
        let stream_id: &str = &Uuid::new_v4().to_string();
        let stream = Stream {
            id: stream_id.to_string(),
            state: StreamState::Running,
            remote_candidates: HashMap::new(),
            local_candidates: HashMap::new(),
        };

        self.streams.insert(stream_id.to_string(), stream);

        stream_id.to_string()
    }

    /// Gather candidates for a particular stream
    pub fn gather_candidates(&mut self, stream_id: &str, component_id: &u16) {
        let stream = self.streams.get(stream_id).unwrap();

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
        let conn = UdpSocket::bind(ipv4_addr).unwrap();

        let port = ipv4_addr.port();

        //let skt = SocketAddr::new(ipv4_addr, port);

        setup_stun_server(conn);
        //let rtp_stream = RtpSession::connect_to(conn, "0.0.0.0:0".parse().unwrap())

        // Get new candidate
        let mut candidate = Candidate {
            conn: ipv4_addr.ip(),
            port: port,
            proto: Proto::Udp,
            foundation: "deadbeef".to_string(),
            component_id: None,
            priority: 0,
            candidate_type: CandidateType::Host,
            rel_addr: None,
            rel_port: None,
        };
        Agent::set_priority_candidate(candidate, *component_id);

        candidates.push(candidate);
    }

    fn pair_candidates(&self) {
        // TODO(tlam): Full implementation only
    }

    fn set_priority_candidate(mut candidate: Candidate, component_id: u16) {
        let priority = ((2^24)*(126) +
                        (2^8)*(65535) + // 65535 from #rfc5245
                        (2^0)*(256 - component_id)) as u32;

        candidate.priority = priority;
    }

    fn set_default_candidate(candidates: Vec<Candidate>) {
        // TODO(tlam): The default candidate should be firs ton que queue,
        //              however there's no process in place right now.
    }
}
