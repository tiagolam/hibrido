use std::thread;
use std::sync::Arc;
use std::collections::BTreeMap;
use std::sync::RwLock;

use nickel;
use rustc_serialize;
use nickel::{Nickel, HttpRouter, Request, Response, MiddlewareResult, JsonBody};
use nickel::status::StatusCode;
use rustc_serialize::json::{Json, ToJson};
use super::handlers;

use convo::convo::{Conference};
use convo::member::{Member};
use sdp::{SessionDescription};

pub struct HttpServer;

#[derive(RustcDecodable, RustcEncodable)]
struct ConferencePost {
    convo_id: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct ConferenceResponse {
    convo_id: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct MemberPost {
    sdp: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct MemberResponse {
    member_id: String,
}

impl ToJson for ConferenceResponse {
    fn to_json(&self) -> Json {
        let mut map = BTreeMap::new();
        map.insert("convo_id".to_string(), self.convo_id.to_json());
        Json::Object(map)
    }
}

fn post_conference<'mw>(req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {

    // Parse JSON
    let convo_post = req.json_as::<ConferencePost>().unwrap();

    // Create new convo or return an alrady existing one
    let convo = Conference::new(&convo_post.convo_id);

    let response = ConferenceResponse {
        convo_id: convo.id.to_string(),
    };
 
    // Compose response
    res.send(response.to_json())
}

fn get_conference<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {

    let convoid = req.param("convoid").unwrap();

    // Get the convo of id :convoid
    let convo = Conference::get(convoid);

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

fn post_member<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {

    let mut convo;
    {
        let convoid = req.param("convoid").unwrap();

        // Try and find convo
        let convo_result = Conference::get(convoid);
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

    // Add member / SDP to the convo, negotiating the SDPs
    let sdp_answer = convo.add_member(member.clone());

    // Return any response errors, like the negotiating
    // between the SDPs failing, or because the parse
    // failed. 

    debug!("SDP Answer {}", sdp_answer.clone().unwrap().to_string());

    // TODO(tlam): One thread per connection? Ugh...
    thread::spawn(move || {
        loop {
            convo.process_engine(member.clone());
        }
    });

    res.send(sdp_answer.clone().unwrap().to_string().as_bytes())
}

fn get_member<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {

    let convoid = req.param("convoid").unwrap();

    // Get the convo of id :convoid
    let convo = Conference::get(convoid);
    if !convo.is_some() {
        res.set(StatusCode::NotFound);
        return res.render("Conference {} not found", &convoid)
    }

    res.send("Conference found")
    // Find member
}

impl handlers for HttpServer {

    fn start_server() {
        let mut server = Nickel::new();

        // Convo related operations
        server.post("/convo", post_conference);
        server.get("/convo/:convoid", get_conference);
        server.post("/convo/:convoid/member", post_member);
        server.get("/convo/:convoid/member/:memberid", get_member);
        // Member related operations
        server.get("/member/:memberid", get_member);

        /*server.get("/user/:userid", middleware! { |request|
            format!("This is user: {:?}", request.param("userid"))
        });*/

        server.listen("127.0.0.1:3080");
    }
}

