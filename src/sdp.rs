use std::str::FromStr;
use std::string::ToString;
use std::net::IpAddr;

#[derive(Clone, Debug)]
pub struct Origin {
    pub username: String,
    pub session_id: String,
    pub session_version: u64,
    pub net_type: NetType,
    pub addr_type: AddrType,
    pub ip_address: IpAddr,
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub net_type: NetType,
    pub addr_type: AddrType,
    pub ip_address: IpAddr,
    pub ttl: u8,
    pub nr_addrs: u8,
}

#[derive(Clone, Debug)]
pub struct Timing {
    pub start_time: u64,
    pub stop_time: u64,
}

#[derive(Clone, Debug)]
pub struct Attr {
    pub name: String,
    pub value: String,
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
    //conn:
    //bandwidth:
    //encrypt_key:
    pub attrs: Vec<Attr>
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

impl FromStr for NetType {
    type Err = ();

    fn from_str(s: &str) -> Result<NetType, ()> {
        match s {
            "IN" => Ok(NetType::IN),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MediaProto {
    Udp,
    RtpAvp,
    RtpSavp,
}

impl ToString for MediaProto {

    fn to_string(&self) -> String {
        match *self {
        self::MediaProto::Udp => "UDP".to_string(),
        self::MediaProto::RtpAvp => "RTP/AVP".to_string(),
        self::MediaProto::RtpSavp => "RTP/SAVP".to_string(),
        }
    } 
}

impl FromStr for MediaProto {
    type Err = ();

    fn from_str(s: &str) -> Result<MediaProto, ()> {
        match s {
            "UDP" => Ok(MediaProto::Udp),
            "RTP/AVP" => Ok(MediaProto::RtpAvp),
            "RTP/SAVP" => Ok(MediaProto::RtpSavp),
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

#[derive(Clone, Debug)]
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
            self::MediaType::AUDIO => "audio".to_string(),
            self::MediaType::VIDEO => "video".to_string(),
            self::MediaType::TEXT => "text".to_string(),
            self::MediaType::APPLICATION => "application".to_string(),
            self::MediaType::MESSAGE => "message".to_string(),
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
            println!("line: {}", line);
            if let Some(parsed) = parse_line(line) {
                match sdm {
                    None => {
                        println!("sdm is None");
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
                       println!("sdm has \"ref media\"");
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
                println!("invalid: {}", line);
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
                println!("v => {}", v);
                Some(SdpLine::ProtocolVersion(v))
            } else {
                println!("v is None");
                None
            }
        },
        "o" => {
            if let Some(o) = parse_origin(line_val) {
                Some(SdpLine::Origin(o))
            } else {
                println!("o is None");
                None
            }
        },
        "s" => {
            if let Some(s) = parse_session_name(line_val) {
                Some(SdpLine::Description(s))
            } else {
                println!("Session name not valid");
                None
            }
        },
        "i" => {
            if let Some(i) = parse_information(line_val) {
                Some(SdpLine::Information(i))
            } else {
                println!("Information not valid");
                None
            }
        },
        // TODO: support optional URI, "uri"
        // TODO: support optional email address and phone number, "e" and "p"
        "c" => {
            if let Some(c) = parse_connection(line_val) {
                Some(SdpLine::Connection(c))
            } else {
                println!("Connection not valid");
                None
            }
        },
        "t" => {
            if let Some(t) = parse_timing(line_val) {
                Some(SdpLine::Timing(t))
            } else {
                println!("Timing not valid");
                None
            }
        },
        "a" => {
            if let Some(a) = parse_attr(line_val) {
                Some(SdpLine::Attr(a))
            } else {
                println!("Attribute not valid");
                None
            }        
        },
        "m" => {
            if let Some(m) = parse_media(line_val) {
                Some(SdpLine::Media(m))
            } else {
                println!("Media not valid");
                None
            }        
        },
         _ => None
    }

}

impl ToString for SessionDescription {

    fn to_string(&self) -> String {
        let mut session_description = format!("v={}\n
                 o={} {} {} {:?} {:?} {}\n
                 s={}\n
                 i={}\n
                 c={:?} {:?} {}\n",
                 self.ver.unwrap(),
                 self.origin.clone().unwrap().username,
                 self.origin.clone().unwrap().session_id,
                 self.origin.clone().unwrap().session_version,
                 self.origin.clone().unwrap().net_type,
                 self.origin.clone().unwrap().addr_type,
                 self.origin.clone().unwrap().ip_address,
                 self.name.clone().unwrap(),
                 self.info.clone().unwrap(),
                 self.conn.clone().unwrap().net_type,
                 self.conn.clone().unwrap().addr_type,
                 self.conn.clone().unwrap().ip_address);

        for i in 0..self.attrs.len() {
            let session_attrs = format!("a={}:{}\n",
                     self.attrs[i].name,
                     self.attrs[i].value);

            session_description = session_description + &session_attrs;
        }

        session_description = session_description + &format!("t={} {}\n",
                 self.timing.clone().unwrap().start_time,
                 self.timing.clone().unwrap().stop_time);

        for i in 0..self.media.len() {
            let media_description = format!("m={} {} {} {}\n",
                 self.media[i].media.media.to_string(),
                 self.media[i].media.port,
                 self.media[i].media.proto.to_string(),
                 self.media[i].media.fmt[0]);

            session_description = session_description + &media_description;
        }

        session_description
    } 
}

pub fn negotiate_with(sdp_orig: Option<&SessionDescription>, sdp_offer: &SessionDescription) -> SessionDescription {

    //TODO Refine negotiation based on RFC#3264
    //     We are only returning the offer as is now

    // Construct a new SDP based on the negotiation
    sdp_offer.clone()
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

    println!("information: {}", information);

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
                println!("Ipv4 address")
            }
            IpAddr::V6(_) => {
                is_mulcast_or_ipv6 = true;
                println!("Ipv6 address")
            }
        },
        Err(_) => return None,
    }

    let mut ttl:u8 = 0;
    let mut nr_addrs:u8 = 0;
    if is_mulcast_or_ipv6 && conn_addr.len() > 1 {
        // TODO(tlam) Log it instead of println!
        println!("Maformed connection address");
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
    let parts = text.split(' ').collect::<Vec<&str>>();
    if parts.len() != 1 {
        return None;
    }

    let attrs = parts[0].split(':').collect::<Vec<&str>>();
    if attrs.len() == 1 {
        Some(Attr {
            name: attrs[0].to_string(),
            value: "".to_string(),
        })
    } else {
        Some(Attr {
            name: attrs[0].to_string(),
            value: attrs[1].to_string(),
        })
    }
}

fn parse_media(text: &str) -> Option<Media> {
    let parts = text.split(' ').collect::<Vec<&str>>();
    if parts.len() < 4 {
        return None;
    }

    let fmt:Vec<String> = parts[0..parts.len()-3].iter().map(|s| s.to_string()).collect();

    if fmt.len() == 0 {
        return None;
    }

    Some(Media {
        media: parts[0].parse::<MediaType>().unwrap(),
        port: parts[1].parse::<u16>().unwrap(),
        proto: parts[2].parse::<MediaProto>().unwrap(),
        fmt: fmt,
    })
}

