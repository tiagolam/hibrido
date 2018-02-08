use std::str::FromStr;
use std::string::ToString;
use std::net::IpAddr;

use ice;

#[derive(Clone, Debug)]
pub struct Origin {
    pub username: String,
    pub session_id: String,
    pub session_version: u64,
    pub net_type: NetType,
    pub addr_type: AddrType,
    pub ip_address: IpAddr,
}

impl ToString for Origin {

    fn to_string(&self) -> String {
        format!("o={} {} {} {} {} {}\n",
                self.username,
                self.session_id,
                self.session_version,
                self.net_type.to_string(),
                self.addr_type.to_string(),
                self.ip_address)
    }
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub net_type: NetType,
    pub addr_type: AddrType,
    pub ip_address: IpAddr,
    pub ttl: u8,
    pub nr_addrs: u8,
}

impl ToString for Connection {

    fn to_string(&self) -> String {
        format!("c={} {} {} {} {}\n",
            self.net_type.to_string(),
            self.addr_type.to_string(),
            self.ip_address,
            self.ttl,
            self.nr_addrs)
    }
}

#[derive(Clone, Debug)]
pub struct Timing {
    pub start_time: u64,
    pub stop_time: u64,
}

#[derive(Clone, Debug)]
pub struct PTimeValue {
    value: u32,
}

impl PartialEq for PTimeValue {
    fn eq(&self, other: &PTimeValue) -> bool {
        self.value == other.value
    }
}

#[derive(Clone, Debug)]
pub struct RtpMapValue {
    payload_type: u32,
    encoding_name: String,
    clock_rate: u32,
    encoding_params: Option<u32>,
}

impl FromStr for RtpMapValue {
    type Err = ();

    fn from_str(rtpmap_value: &str) -> Result<Self, Self::Err> {
        let values = rtpmap_value.split(' ').collect::<Vec<&str>>();
        if values.len() != 2 {
            debug!("Invalid value for rtpmap");
            return Err(())
        }

        let payload_type = values[0].parse::<u32>().unwrap();
        let params = values[1].split('/').collect::<Vec<&str>>();

        if params.len() < 2 {
            debug!("Invalid encoding/rate for rtpmap");
            return Err(())
        }

        let encoding_name = FromStr::from_str(params[0]).unwrap();
        let clock_rate = params[1].parse::<u32>().unwrap();

        let mut encoding_params = None;
        if params.len() == 3 {
            encoding_params = params[2].parse::<u32>().ok();
        }

        Ok(RtpMapValue {
            payload_type: payload_type,
            encoding_name: encoding_name,
            clock_rate: clock_rate,
            encoding_params: encoding_params,
        })
    }
}

impl ToString for RtpMapValue {

    fn to_string(&self) -> String {

        let value = format!("{} {}/{}", self.payload_type, self.encoding_name, self.clock_rate);

        match self.encoding_params {
            Some(params) => { format!("{}/{}", value, params) },
            None => { value  },
        }
    }
}

impl PartialEq for RtpMapValue {

    fn eq(&self, other: &RtpMapValue) -> bool {
        self.payload_type == other.payload_type &&
            self.encoding_name == other.encoding_name &&
            self.clock_rate == other.clock_rate
    }
}

#[derive(Clone, Debug)]
pub struct FmtPValue {
    format: u32,
    format_params: String,
}

impl FromStr for FmtPValue {
    type Err = ();

    fn from_str(fmtp_value: &str) -> Result<Self, Self::Err> {
        let values = fmtp_value.split(' ').collect::<Vec<&str>>();
        if values.len() != 2 {
            debug!("Invalid value for fmtp");
            return Err(())
        }

        let format = values[0].parse::<u32>().unwrap();
        let fmt_params = values[1].to_string();

        Ok(FmtPValue {
            format: format,
            format_params: fmt_params,
        })
    }
}

impl ToString for FmtPValue {

    fn to_string(&self) -> String {
        format!("{} {}", self.format, self.format_params)
    }
}

impl PartialEq for FmtPValue {

    fn eq(&self, other: &FmtPValue) -> bool {
        self.format == other.format &&
            self.format_params == self.format_params
    }
}

#[derive(Clone, Debug)]
pub struct CandidateValue {
    pub ice_candidate: ice::Candidate,
}

impl FromStr for CandidateValue {
    type Err = ();

    fn from_str(candidate_value: &str) -> Result<Self, Self::Err> {
        let values = candidate_value.split(' ').collect::<Vec<&str>>();
        if values.len() < 8 {
            debug!("Invalid value for candidate");
            return Err(())
        }

        let foundation = values[0].to_string();
        let component_id = values[1].parse::<u16>().unwrap();
        let proto = values[2].parse::<ice::Proto>().unwrap();
        let priority = values[3].parse::<u32>().unwrap();
        let conn = FromStr::from_str(values[4]).unwrap();
        let port = values[5].parse::<u16>().unwrap();
        let typ = values[6].to_string();
        if typ != "typ" {
            debug!("Invalid value for candidate, no 'typ' found");
            return Err(())
        }
        let candidate_type = values[7].parse::<ice::CandidateType>().unwrap();

        let mut rel_addr = None;
        let mut rel_port = None;
        if values.len() >= 12 {
            let raddr = values[8].to_string();
            if raddr != "raddr" {
                debug!("No 'raddr' found");
            } else {
                rel_addr = Some(FromStr::from_str(values[9]).unwrap());
            }

            let rport = values[10].to_string();
            if rport != "rport" {
                debug!("No 'rport' found");
            } else {
                rel_port = Some(values[11].parse::<u16>().unwrap());
            }
        }

        Ok(CandidateValue {
            ice_candidate: ice::Candidate {
                conn: conn,
                port: port,
                proto: proto,
                foundation: foundation,
                component_id: Some(component_id),
                priority: priority,
                candidate_type: candidate_type,
                rel_addr: rel_addr,
                rel_port: rel_port,
            }
        })
    }
}

impl ToString for CandidateValue {

    fn to_string(&self) -> String {

        let value = format!("{} {} {} {} {} {} typ {}", self.ice_candidate.foundation, self.ice_candidate.component_id.unwrap(), self.ice_candidate.proto.to_string(), self.ice_candidate.priority, self.ice_candidate.conn.to_string(), self.ice_candidate.port, self.ice_candidate.candidate_type.to_string());

        match self.ice_candidate.rel_addr {
            Some(rel_addr) => { format!("{} raddr {:?} rport {:?}", value, rel_addr, self.ice_candidate.rel_port) },
            None => { value },
        }
    }
}

impl PartialEq for CandidateValue {
    fn eq(&self, other: &CandidateValue) -> bool {
        true
        //self.ice_candidate == other.ice_candidate
    }
}


#[derive(Clone, Debug)]
pub struct IceUfragValue {
    value: String,
}

impl PartialEq for IceUfragValue {
    fn eq(&self, other: &IceUfragValue) -> bool {
        self.value == other.value
    }
}

#[derive(Clone, Debug)]
pub struct IcePwdValue {
    value: String,
}

impl PartialEq for IcePwdValue {
    fn eq(&self, other: &IcePwdValue) -> bool {
        self.value == other.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attr {
    SendRecv,
    SendOnly,
    RecvOnly,
    Inactive,
    PTime(PTimeValue),
    RtpMap(RtpMapValue),
    FmtP(FmtPValue),
    Candidate(CandidateValue),
    IceUfrag(IceUfragValue),
    IcePwd(IcePwdValue),
    IceMismatch,
    IceLite,
}

impl ToString for Attr {

    fn to_string(&self) -> String {

        let mut name;
        let mut value: Option<String> = None;
        match *self {
            Attr::RecvOnly => {
                name = "recvonly".to_string();
            },
            Attr::SendOnly => {
                name = "sendonly".to_string();
            },
            Attr::SendRecv => {
                name = "sendrecv".to_string();
            },
            Attr::Inactive => {
                name = "inactive".to_string();
            },
            Attr::PTime(PTimeValue{value: x}) => {
                name = "ptime".to_string();
                value = Some(x.to_string());
            },
            Attr::RtpMap(ref x) => {
                name = "rtpmap".to_string();
                value = Some(x.to_string());
            },
            Attr::FmtP(ref x) => {
                name = "fmtp".to_string();
                value = Some(x.to_string());
            },
            Attr::Candidate(ref x) => {
                name = "candidate".to_string();
                value = Some(x.to_string());
            },
            Attr::IceUfrag(IceUfragValue{value: ref x}) => {
                name = "ice-ufrag".to_string();
                value = Some(x.to_string());
            },
            Attr::IcePwd(IcePwdValue{value: ref x}) => {
                name = "ice-pwd".to_string();
                value = Some(x.to_string());
            },
            Attr::IceMismatch => {
                name = "ice-mismatch".to_string();
            },
            Attr::IceLite => {
                name = "ice-lite".to_string();
            },
        }

        match value {
            Some(x) => format!("a={}:{}\n", name, x),
            None => format!("a={}\n", name)
        }
    }
}

trait AttrFromStr {
    fn from_str(attr_type: &str, attr_value: Option<&str>) -> Result<Attr, ()>;
}

impl AttrFromStr for Attr {

    fn from_str(attr_type: &str, attr_value: Option<&str>) -> Result<Attr, ()> {
        match attr_type {
            "recvonly"  => Ok(Attr::RecvOnly),
            "sendonly"  => Ok(Attr::SendOnly),
            "sendrecv"  => Ok(Attr::SendRecv),
            "inactive"  => Ok(Attr::Inactive),
            "ptime"     => {
                Ok(Attr::PTime(PTimeValue{
                    value: attr_value.unwrap().parse::<u32>().unwrap()
                }))
            },
            "rtpmap"    => {
                Ok(Attr::RtpMap(
                    attr_value.unwrap().parse::<RtpMapValue>().unwrap()
                ))
            },
            "fmtp"      => {
                Ok(Attr::FmtP(
                    attr_value.unwrap().parse::<FmtPValue>().unwrap()
                ))
            },
            "candidate" => {
                Ok(Attr::Candidate(
                    attr_value.unwrap().parse::<CandidateValue>().unwrap()
                ))
            },
            "ice-ufrag" => {
                Ok(Attr::IceUfrag(IceUfragValue{
                    value: attr_value.unwrap().to_string(),
                }))
            },
            "ice-pwd" => {
                Ok(Attr::IcePwd(IcePwdValue{
                    value: attr_value.unwrap().to_string(),
                }))
            },
            "ice-mismatch"  => Ok(Attr::IceMismatch),
            "ice-lite"  => Ok(Attr::IceLite),
            _           => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionData {
    pub ip_address: IpAddr,
    pub ttl: Option<u8>,
    pub num_addresses: Option<u8>
}

#[derive(Clone, Debug)]
pub struct SessionDescription {
    pub ver: Option<i32>, 
    pub origin: Option<Origin>,
    pub name: Option<String>,
    pub info: Option<String>,
    pub uri: Option<String>,
    //email:
    //phone:
    pub conn: Option<Connection>,
    //bandwidth: 
    pub timing: Option<Timing>,
    //time_zones:
    //encrypt_key:
    pub attrs: Vec<Attr>,
    pub media: Vec<MediaDescription>,
}

//struct TimeDescription {
//    timing:
//    repeat_times:
//}

#[derive(Clone, Debug)]
pub struct MediaDescription {
    pub media: Media,
    //title:
    //bandwidth:
    //encrypt_key:
    pub attrs: Vec<Attr>
}


impl ToString for MediaDescription {

    fn to_string(&self) -> String {
        let mut media_description = format!("m={} {} {}",
            self.media.media.to_string(),
            self.media.port,
            self.media.proto.to_string());

        for i in 0..self.media.fmt.len() {
            media_description = format!("{} {}", media_description, self.media.fmt[i]);
        }

        media_description = format!("{}\n", media_description);

        for i in 0..self.attrs.len() {
            let media_attrs = format!("{}",
                self.attrs[i].to_string());

            media_description = media_description + &media_attrs;
        }

        media_description
    }
}

impl MediaDescription {
    pub fn new(media: Media) -> MediaDescription {
        MediaDescription {
            media: media,
            attrs: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParseResult {
    pub desc: SessionDescription,
    pub ignored_lines: Vec<SdpLine>,
    pub unparsed_lines: Vec<String>,
}

impl ParseResult {
    pub fn new() -> ParseResult {
        ParseResult {
            desc: SessionDescription::new(),
            ignored_lines: vec![],
            unparsed_lines: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub enum SdpLine {
    ProtocolVersion(i32),
    Origin(Origin),
    Description(String),
    Information(String),
    Connection(Connection),
    Timing(Timing),
    Attr(Attr),
    Media(Media),
}

#[derive(Clone, Debug)]
pub enum AddrType {
    IP4,
    IP6,
}

impl ToString for AddrType {

    fn to_string(&self) -> String {
        match *self {
            AddrType::IP4 => return "IP4".to_string(),
            AddrType::IP6 => return "IP6".to_string(),
        }
    }
}

impl FromStr for AddrType {
    type Err = ();

    fn from_str(s: &str) -> Result<AddrType, ()> {
        match s {
            "IP4" => Ok(AddrType::IP4),
            "IP6" => Ok(AddrType::IP6),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum NetType {
    IN,
}

impl ToString for NetType {

    fn to_string(&self) -> String {
        match *self {
            NetType::IN => "IN".to_string(),
        }
    }
}

impl FromStr for NetType {
    type Err = ();

    fn from_str(s: &str) -> Result<NetType, ()> {
        match s {
            "IN" => Ok(NetType::IN),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MediaProto {
    Udp,
    RtpAvp,
    RtpAvpf,
    RtpSavp,
    RtpSavpf,
    UdpTlsRtpSavpf,
}

impl ToString for MediaProto {

    fn to_string(&self) -> String {
        match *self {
            MediaProto::Udp => "UDP".to_string(),
            MediaProto::RtpAvp => "RTP/AVP".to_string(),
            MediaProto::RtpAvpf => "RTP/AVPF".to_string(),
            MediaProto::RtpSavp => "RTP/SAVP".to_string(),
            MediaProto::RtpSavpf => "RTP/SAVPF".to_string(),
            MediaProto::UdpTlsRtpSavpf => "UDP/TLS/RTP/SAVPF".to_string(),
        }
    } 
}

impl FromStr for MediaProto {
    type Err = ();

    fn from_str(s: &str) -> Result<MediaProto, ()> {
        match s {
            "UDP" => Ok(MediaProto::Udp),
            "RTP/AVP" => Ok(MediaProto::RtpAvp),
            "RTP/AVPF" => Ok(MediaProto::RtpAvpf),
            "RTP/SAVP" => Ok(MediaProto::RtpSavp),
            "RTP/SAVPF" => Ok(MediaProto::RtpSavpf),
            "UDP/TLS/RTP/SAVPF" => Ok(MediaProto::UdpTlsRtpSavpf),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Media {
    pub media: MediaType,
    pub port: u16,
    pub proto: MediaProto,
    pub fmt: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MediaType {
    AUDIO,
    VIDEO,
    TEXT,
    APPLICATION,
    MESSAGE,
}

impl ToString for MediaType {

    fn to_string(&self) -> String {
        match *self {
            MediaType::AUDIO => "audio".to_string(),
            MediaType::VIDEO => "video".to_string(),
            MediaType::TEXT => "text".to_string(),
            MediaType::APPLICATION => "application".to_string(),
            MediaType::MESSAGE => "message".to_string(),
        }
    } 
}

impl FromStr for MediaType {
    type Err = ();

    fn from_str(s: &str) -> Result<MediaType, ()> {
        match s {
            "audio" => Ok(MediaType::AUDIO),
            "video" => Ok(MediaType::VIDEO),
            "text" => Ok(MediaType::TEXT),
            "application" => Ok(MediaType::APPLICATION),
            "message" => Ok(MediaType::MESSAGE),
            _ => Err(()),
        }
    }
}

impl SessionDescription {

    pub fn new() -> SessionDescription {
        SessionDescription {
            ver: None,
            origin: None,
            name: None,
            info: None,
            uri: None,
            conn: None,
            timing: None,
            attrs: vec![],
            media: vec![],
        }
    }

    pub fn from_sdp(&self, sdp: &str) -> ParseResult {
        let mut res = ParseResult::new();
        let sdm: Option<MediaDescription> = None;
        let mut prev_media: Option<MediaDescription> = None;
        let mut first_media = false;

        for mut line in sdp.lines() {
            line = line.trim();
            debug!("line: {}", line);
            if let Some(parsed) = parse_line(line) {
                match sdm {
                    None => {
                        debug!("sdm is None");
                        match parsed {
                            SdpLine::ProtocolVersion(v) => { res.desc.ver = Some(v); },
                            SdpLine::Origin(o) => { res.desc.origin = Some(o); },
                            SdpLine::Description(s) => { res.desc.name = Some(s); },
                            SdpLine::Information(i) => { res.desc.info = Some(i); },
                            SdpLine::Connection(c) => { res.desc.conn = Some(c); },
                            SdpLine::Timing(t) => { res.desc.timing = Some(t); },
                            SdpLine::Attr(a) => {
                                if first_media {
                                    let size = res.desc.media.len()-1;
                                    res.desc.media[size].attrs.push(a);
                                } else {
                                    res.desc.attrs.push(a);
                                }
                            },
                            SdpLine::Media(m) => {
                                res.desc.media.push(MediaDescription::new(m));
                                first_media = true;
                            },
                        }
                    }, Some(_) => {
                       debug!("sdm has \"ref media\"");
                       match parsed {
                            SdpLine::ProtocolVersion(_) => { res.ignored_lines.push(parsed.clone()); },
                            SdpLine::Origin(_) => { res.ignored_lines.push(parsed.clone()); },

                            SdpLine::Description(_) => { res.ignored_lines.push(parsed.clone()); },
                            SdpLine::Information(_) => { res.ignored_lines.push(parsed.clone()); },
                            SdpLine::Connection(_) => { res.ignored_lines.push(parsed.clone()); },
                            SdpLine::Timing(_) => { res.ignored_lines.push(parsed.clone()); },
                            SdpLine::Attr(_) => { res.ignored_lines.push(parsed.clone()); },
                            SdpLine::Media(_) => {
                                prev_media = None;
                                res.ignored_lines.push(parsed.clone());
                            },
                        }
                    }
                }
            } else {
                debug!("invalid: {}", line);
                res.unparsed_lines.push(line.to_string());
            }
        }

        res
    }
}

fn parse_line(line: &str) -> Option<SdpLine> {
    let parts = line.splitn(2, '=').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return None;
    }

    let line_type = parts[0];
    let line_val = parts[1];

    match line_type {
        "v" => {
            if let Ok(v) = FromStr::from_str(line_val) {
                debug!("v => {}", v);
                Some(SdpLine::ProtocolVersion(v))
            } else {
                debug!("v is None");
                None
            }
        },
        "o" => {
            if let Some(o) = parse_origin(line_val) {
                Some(SdpLine::Origin(o))
            } else {
                debug!("o is None");
                None
            }
        },
        "s" => {
            if let Some(s) = parse_session_name(line_val) {
                Some(SdpLine::Description(s))
            } else {
                debug!("Session name not valid");
                None
            }
        },
        "i" => {
            if let Some(i) = parse_information(line_val) {
                Some(SdpLine::Information(i))
            } else {
                debug!("Information not valid");
                None
            }
        },
        // TODO: support optional URI, "uri"
        // TODO: support optional email address and phone number, "e" and "p"
        "c" => {
            if let Some(c) = parse_connection(line_val) {
                Some(SdpLine::Connection(c))
            } else {
                debug!("Connection not valid");
                None
            }
        },
        "t" => {
            if let Some(t) = parse_timing(line_val) {
                Some(SdpLine::Timing(t))
            } else {
                debug!("Timing not valid");
                None
            }
        },
        "a" => {
            if let Some(a) = parse_attr(line_val) {
                Some(SdpLine::Attr(a))
            } else {
                debug!("Attribute not valid");
                None
            }        
        },
        "m" => {
            if let Some(m) = parse_media(line_val) {
                Some(SdpLine::Media(m))
            } else {
                debug!("Media not valid");
                None
            }        
        },
         _ => None
    }

}

impl ToString for SessionDescription {

    fn to_string(&self) -> String {
        let mut session_description = format!("v={}\n{}s={}\n{}",
                 self.ver.unwrap(),
                 self.origin.clone().unwrap().to_string(),
                 self.name.clone().unwrap(),
                 //self.info.clone().unwrap(),
                 self.conn.clone().unwrap().to_string());

        session_description = session_description + &format!("t={} {}\n",
                 self.timing.clone().unwrap().start_time,
                 self.timing.clone().unwrap().stop_time);

        for i in 0..self.attrs.len() {
            session_description = session_description + &self.attrs[i].to_string();
        }

        for i in 0..self.media.len() {
            session_description = session_description + &self.media[i].to_string();
        }

        session_description
    } 
}

fn parse_origin(text: &str) -> Option<Origin> {
    let parts = text.split(' ').collect::<Vec<&str>>();
    if parts.len() != 6 {
        return None;
    }

    let session_version = FromStr::from_str(parts[2]);
    let ip_addr = FromStr::from_str(parts[5]);

    if session_version.is_err() || ip_addr.is_err() {
        return None
    }

    Some(Origin {
        username: parts[0].to_string(),
        session_id: parts[1].to_string(),
        session_version: session_version.unwrap(),
        net_type: parts[3].parse::<NetType>().unwrap(),
        addr_type: parts[4].parse::<AddrType>().unwrap(),
        ip_address: ip_addr.unwrap(),
    })
}

fn parse_session_name(text: &str) -> Option<String> {

    // TODO(tiagolam) Validate strings according to the spec

    let session_name = text.to_string();

    debug!("session_name: {}", session_name);

    Some(session_name)
}

fn parse_information(text: &str) -> Option<String> {

    // TODO(tiagolam) Validate strings according to the spec

    let information = text.to_string();

    debug!("information: {}", information);

    Some(information)
}

fn parse_connection(text: &str) -> Option<Connection> {
    let parts = text.split(' ').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return None;
    }

    let conn_addr = parts[2].to_string();
    let conn_addr = conn_addr.split('/').collect::<Vec<&str>>();

    let ip_addr = FromStr::from_str(conn_addr[0]);

    //let ip_addr::IpAddr = ip_addr.unwrap();

    let mut is_mulcast_or_ipv6:bool = false;

    match ip_addr {
        Ok(value) => match value {
            IpAddr::V4(x) => {
                is_mulcast_or_ipv6 = x.is_multicast();
                debug!("Ipv4 address")
            }
            IpAddr::V6(_) => {
                is_mulcast_or_ipv6 = true;
                debug!("Ipv6 address")
            }
        },
        Err(_) => return None,
    }

    let mut ttl:u8 = 0;
    let mut nr_addrs:u8 = 0;
    if is_mulcast_or_ipv6 && conn_addr.len() > 1 {
        debug!("Malformed connection address");
        return None;
    } else if conn_addr.len() == 3 {
        ttl = conn_addr[1].parse::<u8>().unwrap();
        nr_addrs = conn_addr[2].parse::<u8>().unwrap();
    } else if conn_addr.len() == 2 {
        ttl = conn_addr[1].parse::<u8>().unwrap();
    }

    Some(Connection {
        // TODO(tlam) Deal with errors instead of using unwrap
        net_type: parts[0].parse::<NetType>().unwrap(),
        addr_type: parts[1].parse::<AddrType>().unwrap(),
        ip_address: ip_addr.unwrap(),
        ttl: ttl,
        nr_addrs: nr_addrs,
    })
}

fn parse_timing(text: &str) -> Option<Timing> {
    let parts = text.split(' ').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return None;
    }

    // TODO(tlam) Verify this is NTP timestamps
    let start_time:u64 = parts[0].parse::<u64>().unwrap();
    let stop_time:u64 = parts[1].parse::<u64>().unwrap();

    Some(Timing {
        start_time: start_time,
        stop_time: stop_time,
    })
}

fn parse_attr(text: &str) -> Option<Attr> {
    let parts = text.split(':').collect::<Vec<&str>>();

    let result;
    match parts.len() {
        1 => {
            result = Attr::from_str(parts[0], None)
        },
        2 => {
            result = Attr::from_str(parts[0], Some(parts[1]))
        },
        _ => {
            debug!("Invalid attribute");
            result = Err(())
        },
    }

    result.ok()
}

fn parse_media(text: &str) -> Option<Media> {
    let parts = text.split(' ').collect::<Vec<&str>>();
    if parts.len() < 4 {
        return None;
    }

    let fmt:Vec<String> = parts[3..].iter().map(|s| s.to_string()).collect();

    if fmt.len() == 0 {
        return None;
    }

    debug!("Fmt items {:?}", fmt);

    Some(Media {
        media: parts[0].parse::<MediaType>().unwrap(),
        port: parts[1].parse::<u16>().unwrap(),
        proto: parts[2].parse::<MediaProto>().unwrap(),
        fmt: fmt,
    })
}

fn negotiate_media_stream(orig_media: MediaDescription, offer_media: &mut MediaDescription) -> bool {

    if orig_media.media.media != offer_media.media.media {
        debug!("Different media types [orig: {}, offer: {}", orig_media.media.media.to_string(), offer_media.media.media.to_string());
        return false
    }

    if offer_media.media.proto != MediaProto::RtpAvp {
        debug!("Media proto {} not supported", offer_media.media.proto.to_string());
        return false
    }

    let mut found_match = true;
    let offer_fmt =  offer_media.media.fmt.clone();
    let result = offer_fmt.into_iter().filter(|x| orig_media.media.fmt.contains(x)).collect::<Vec<_>>();
    //offer_media.media.fmt.retain(|x| !orig_media.media.fmt.contains(x));

    if result.len() == 0 {
        debug!("Media {} and {} are not a match", orig_media.to_string(), offer_media.to_string());
        return false
    }

    debug!("Media {} and {} are a match: {}", orig_media.to_string(), offer_media.to_string(), result.len());

    offer_media.media.fmt = result;

    let mut filtered_attrs = vec![];
    let offer_fmts = offer_media.media.fmt.clone();
    for fmt in offer_fmts {
        let value = fmt.parse::<u32>().unwrap();

        let mut orig_attrs = orig_media.attrs.clone();
        for orig_attr in orig_attrs.drain(0..) {
            match orig_attr {
                Attr::RtpMap(RtpMapValue{payload_type: x, ..}) => {
                    if value != x {
                        debug!("Different media fmt [orig: {}, offer: {}]", value, x);
                        continue;
                    }

                    filtered_attrs.push(orig_attr);
                },
                _ => {},
            }
        }
    }

    let offer_attrs = filtered_attrs.clone();
    let result = offer_attrs.into_iter().filter(|x| orig_media.attrs.contains(x)).collect::<Vec<_>>();

    if result.len() == 0 {
        debug!("attrs {:?} and {:?} are not a match", filtered_attrs, orig_media.attrs);
        return false
    }

    debug!("Media is a match: {}", result[0].to_string());

    let mut offer_media_attrs = offer_media.attrs.clone();
    offer_media.attrs = result;
    // For each of the attributes present on the offer, negotiate, and put the
    // result on the answer
    for offer_attr in &mut offer_media_attrs {
        match *offer_attr {
            Attr::SendOnly => {
                offer_media.attrs.push(Attr::RecvOnly)
            },
            Attr::RecvOnly => {
                offer_media.attrs.push(Attr::SendOnly)
            },
            Attr::SendRecv => {
                offer_media.attrs.push(Attr::SendRecv)
            },
            Attr::Inactive => {
                offer_media.attrs.push(Attr::Inactive)
            },
            Attr::PTime(ref x) => {
                offer_media.attrs.push(Attr::PTime(x.clone()))
            },
            Attr::IceUfrag(ref x) => {
                // Generate ufrag and pass
                offer_media.attrs.push(Attr::IceUfrag(IceUfragValue {
                    value: "Oyef7uvBlwafI3hT".to_string()
                }));

                offer_media.attrs.push(Attr::IcePwd(IcePwdValue {
                    value: "T0teqPLNQQOf+5W+ls+P2p16".to_string()
                }));
            }
            _ => {},
        }
    }

    return found_match
}

fn set_media_stream(offer_media: &mut MediaDescription) {
    let mut offer_media_attrs = offer_media.attrs.clone();
    let mut final_attrs = vec![];
    // For each of the attributes present on the offer, negotiate, and put the
    // result on the answer
    for offer_attr in &mut offer_media_attrs {
        match *offer_attr {
            Attr::SendOnly => {
                final_attrs.push(Attr::RecvOnly)
            },
            Attr::RecvOnly => {
                final_attrs.push(Attr::SendOnly)
            },
            Attr::SendRecv => {
                final_attrs.push(Attr::SendRecv)
            },
            Attr::Inactive => {
                final_attrs.push(Attr::Inactive)
            },
            Attr::PTime(ref x) => {
                final_attrs.push(Attr::PTime(x.clone()))
            },
            Attr::IceUfrag(ref x) => {
                // Generate ufrag and pass
                final_attrs.push(Attr::IceUfrag(IceUfragValue {
                    value: "Oyef7uvBlwafI3hT".to_string()
                }));

                final_attrs.push(Attr::IcePwd(IcePwdValue {
                    value: "T0teqPLNQQOf+5W+ls+P2p16".to_string()
                }));
            }
            Attr::RtpMap(ref x) => {

                final_attrs.push(Attr::RtpMap(x.clone()));
            },
            Attr::FmtP(ref x) => {

                final_attrs.push(Attr::FmtP(x.clone()));
            },
            _ => {},
        }
    }
    offer_media.media.port = 9;

    offer_media.attrs = final_attrs;
}


pub fn negotiate_with(sdp_orig: Option<&SessionDescription>, sdp_offer: &SessionDescription) -> SessionDescription {

    // TODO Negotiation based on RFC#3264

    // Construct a new SDP based on the negotiation
    if sdp_orig.is_some() {
        let mut sdp_answer = sdp_offer.clone();

        for answer_media in &mut sdp_answer.media {
            let mut found_match = false;
            for orig_media in sdp_orig.unwrap().media.iter() {
                found_match = negotiate_media_stream(orig_media.clone(), answer_media);
                if found_match {
                    break
                }
            }

            if !found_match {
                answer_media.media.port = 0;
            }
        }

        // Add "ice-lite" attribute
        sdp_answer.attrs.push(Attr::IceLite);

        // TODO(tlam): Hack that only gets the candidate of the first m=
        for answer_attr in &mut sdp_answer.media[0].attrs {
            match *answer_attr {
                Attr::Candidate(ref x) => {
                    // Set connection
                    let conn = sdp_answer.conn.as_mut().unwrap();
                    conn.ip_address = x.ice_candidate.conn;

                    break;
                },
                _ => {},
            }
        }

        return sdp_answer
    } else {
        let mut sdp_answer = sdp_offer.clone();

        for answer_media in &mut sdp_answer.media {
            set_media_stream(answer_media);
        }

        // Add "ice-lite" attribute
        sdp_answer.attrs.push(Attr::IceLite);

        // TODO(tlam): Hack that only gets the candidate of the first m=
        for answer_attr in &mut sdp_answer.media[0].attrs {
            match *answer_attr {
                Attr::Candidate(ref x) => {
                    // Set connection
                    let conn = sdp_answer.conn.as_mut().unwrap();
                    conn.ip_address = x.ice_candidate.conn;

                    break;
                },
                _ => {},
            }

        }

        return sdp_answer
    }
}
