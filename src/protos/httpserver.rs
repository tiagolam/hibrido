extern crate nickel;

use std::collections::BTreeMap;

use self::nickel::{Nickel, HttpRouter, Request, Response, MiddlewareResult, JsonBody};
use self::nickel::status::StatusCode;
use rustc_serialize::json::{Json, ToJson};
use super::Handlers;

use convo::convo::{Conferences};
use convo::member::{Member};
use sdp::{SessionDescription};

#[derive(RustcDecodable, RustcEncodable)]
pub struct ConferencePost {
    pub convo_id: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct ConferenceResponse {
    pub convo_id: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct MemberPost {
    pub sdp: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct MemberResponse {
    pub member_id: String,
    pub sdp: String,
}

impl ToJson for ConferenceResponse {
    fn to_json(&self) -> Json {
        let mut map = BTreeMap::new();
        map.insert("convo_id".to_string(), self.convo_id.to_json());
        Json::Object(map)
    }
}

impl ToJson for MemberResponse {
    fn to_json(&self) -> Json {
        let mut map = BTreeMap::new();
        map.insert("member_id".to_string(), self.member_id.to_json());
        map.insert("sdp".to_string(), self.sdp.to_json());
        Json::Object(map)
    }
}

pub struct HttpServer {
    convos: Conferences,
}


fn post_conference<'mw>(req: &mut Request<HttpServer>, mut res: Response<'mw, HttpServer>) -> MiddlewareResult<'mw, HttpServer> {
    let handler = req.server_data();
    let convos = &handler.convos;

    // Parse JSON
    let convo_post = req.json_as::<ConferencePost>().unwrap();

    // Create new convo or return an alrady existing one
    let convo = convos.new_convo(&convo_post.convo_id);

    let response = ConferenceResponse {
        convo_id: convo.id.to_string(),
    };

    res.headers_mut().set_raw("Access-Control-Allow-Origin", vec![b"*".to_vec()]);

    // Compose response
    res.send(response.to_json())
}

fn get_conference<'mw>(req: &mut Request<HttpServer>, mut res: Response<'mw, HttpServer>) -> MiddlewareResult<'mw, HttpServer> {
    let handler = req.server_data();
    let convos = &handler.convos;

    let convoid = req.param("convoid").unwrap();

    // Get the convo of id :convoid
    let convo = convos.get_convo(convoid);

    match convo {
        Some(convo) => {
            let response = ConferenceResponse {
                convo_id: convo.id.to_string(),
            };
 
            // Compose response
            res.send(response.to_json())
        },
        None => {
            //res.send("Conference")
            res.set(StatusCode::NotFound);
            res.render("Conference {} not found", &convoid)
        },
    }
}

fn post_member<'mw>(req: &mut Request<HttpServer>, mut res: Response<'mw, HttpServer>) -> MiddlewareResult<'mw, HttpServer> {
    let handler = req.server_data();
    let convos = &handler.convos;

    let convo;
    {
        let convoid = req.param("convoid").unwrap();

        // Try and find convo
        let convo_result = convos.get_convo(convoid);
        if !convo_result.is_some() {
            res.set(StatusCode::NotFound);
            return res.render("Conference {} not found", &convoid)
        }
        convo = convo_result.unwrap();
    }

    // Parse JSON
    let member_post = req.json_as::<MemberPost>().unwrap();
    
    // Create member and insert in convo
    let sdp = SessionDescription::new();
    let parsed_sdp = sdp.from_sdp(&member_post.sdp);
    let member = Member::new(parsed_sdp.desc);

    let memberid  = member.id.clone();

    // Add member / SDP to the convo, negotiating the SDPs
    let sdp_answer = convo.add_member(member);

    // Return any response errors, like the negotiating
    // between the SDPs failing, or because the parse
    // failed. 

    debug!("SDP Answer {}", sdp_answer.clone().unwrap().to_string());

    // Compose response
    let response = MemberResponse {
        member_id: memberid,
        sdp: sdp_answer.unwrap().to_string(),
    };

    res.headers_mut().set_raw("Access-Control-Allow-Origin", vec![b"*".to_vec()]);
 
    res.send(response.to_json())
}

fn get_conference_member<'mw>(req: &mut Request<HttpServer>, mut res: Response<'mw, HttpServer>) -> MiddlewareResult<'mw, HttpServer> {
    let handler = req.server_data();
    let convos = &handler.convos;

    let convoid = req.param("convoid").unwrap();

    // Get the convo of id :convoid
    let convo = convos.get_convo(convoid);
    if !convo.is_some() {
        res.set(StatusCode::NotFound);
        return res.render("Conference {} not found", &convoid)
    }

    // Find member
    let memberid = req.param("memberid").unwrap();
    let member = convo.unwrap().get_member(memberid);
    if !member.is_some() {
        res.set(StatusCode::NotFound);
        return res.send(format!("Member {} not found in conference {}", &memberid, &convoid))
    }

    let member = member.unwrap();
    // Compose response
    let response = MemberResponse {
        member_id: member.id.to_string(),
        sdp: member.sdp.to_string(),
    };
 
    res.send(response.to_json())
}

fn enable_cors<'mw>(_req: &mut Request<HttpServer>, mut res: Response<'mw, HttpServer>) -> MiddlewareResult<'mw, HttpServer> {
    res.headers_mut().set_raw("Access-Control-Allow-Headers", vec![b"content-type".to_vec()]);
    res.headers_mut().set_raw("Access-Control-Allow-Methods", vec![b"POST, OPTIONS".to_vec()]);
    res.headers_mut().set_raw("Access-Control-Allow-Origin", vec![b"*".to_vec()]);
    res.send("")
}

impl Handlers for HttpServer {

    fn start_server(convos: Conferences) {
        let handler = HttpServer {
            convos: convos,
        };
        let mut server = Nickel::with_data(handler);

        // Convo related operations
        server.post("/convo", post_conference);
        server.get("/convo/:convoid", get_conference);
        server.post("/convo/:convoid/member", post_member);
        server.get("/convo/:convoid/member/:memberid", get_conference_member);

        server.utilize(enable_cors);

        match server.listen("127.0.0.1:3080") {
            Err(x) => {
                error!("Failed to start HTTP server {}", x);
                return;
            },
            _ => {}
        }
    }
}
