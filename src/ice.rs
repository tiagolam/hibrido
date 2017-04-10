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
    pub component_id: u16,
    pub priority: u32,
    pub candidate_type: CandidateType,
    pub rel_addr: Option<IpAddr>,
    pub rel_port: Option<u16>,
}

enum IceState {
    Running,
    Completed,
}

enum StreamState {
}

struct Stream {
    id: String,
    state: StreamState,
    nr_components: u8,
    remote_candidates: Vec<Candidates>,
    local_candidates: Vec<Vec<Candidate>>,
}

/*
fn setup_stun_server(conn: SocketAddr) {
    let mut executor = InPlaceExecutor::new().unwrap();

    let spawner = executor.handle();
    let monitor = executor.spawn_monitor(UdpServer::new(conn)
                          .start(spawner.boxed(), BindingHandler::new("T0teqPLNQQOf+5W+ls+P2p16".to_string())));
    let result = executor.run_fiber(monitor).unwrap();
}
*/

pub struct Agent {
    state: IceState,
    start_port: u16,
    streams: HashMap<String, Stream>,
}

static mut START_PORT: u16 = 6000;

impl Agent {
    pub fn new() -> Ice {
        Ice {
            state: IceState::Running,
            start_port: 6000,
            streams: HashMap::new();
        }
    }

    /// Start agent and initiate the regular funtions
    pub fn start() {
    }

    /// Add new stream to the current agent, of the component provided
    pub fn add_stream(&mut self) -> String {
        // Add stream to agent
        let stream_id: &str = &Uuid::new_v4().to_string();
        let stream = Stream {
            id: stream_id.to_owned_string(),
            candidates: Vec::new(),
        }

        self.insert(stream_id.to_owned_string(), stream);

        stream_id.to_owned_string()
    }

    /// Gather candidates for a particular stream
    pub fn gather_strean_candidates(&mut self, strean_id: String, candidate: CandidateType) -> Option<Candidate> {
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

                        break;
                    },
                    _ => {},
                };
            }
        }

        if !ipv4_addr.is_some() {
            return None
        }

        let mut ipv4_addr = ipv4_addr.unwrap();
        unsafe {
            ipv4_addr.set_port(START_PORT);
            START_PORT += 1;
        }
        //let conn = UdpSocket::bind(ipv4_addr).unwrap();

        let port = ipv4_addr.port();

        //let skt = SocketAddr::new(ipv4_addr, port);

        //setup_stun_server(ipv4_addr);
        //let rtp_stream = RtpSession::connect_to(conn, "0.0.0.0:0".parse().unwrap())

        let rtp_priority = ((2^24)*(126) +
                            (2^8)*(65535) + // 65535 from #rfc5245
                            (2^0)*(256 - RTP_COMPONENT_ID)) as u32;


        Some(Candidate {
            conn: ipv4_addr.ip(),
            port: port,
            proto: Proto::Udp,
            foundation: "deadbeef".to_string(),
            component_id: RTP_COMPONENT_ID,
            priority: rtp_priority,
            candidate_type: CandidateType::Host,
            rel_addr: None,
            rel_port: None
        })
    }
}
