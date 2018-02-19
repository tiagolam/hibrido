use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::str;
use std::str::from_utf8;
use std::sync::Arc;

use sdp::{SessionDescription, ParseResult};
use convo::member::{Member};
use convo::convo::{Conferences};
use super::Handlers;

pub struct Tcp {
    convos: Conferences,
}

impl Tcp {
    // We will receive the SDP through TCP
    fn handle_client(&self, mut stream: TcpStream) {
        let mut buf = [0; 1500];
        let _ = stream.read(&mut buf);
        let request = from_utf8(&buf).unwrap();

        debug!("Request on the wire: {:?}", request);

        // Use SDP for negotiating the session

        // 1. Parse the request
        // conference_id: test ...
        // sdp: v=0 ...
        let mut convo_id:&str = "invalid convo";
        let mut raw_sdp:&str = "invalid sdp";
        let mut parsed_len = 0;
        for mut line in request.lines() {
            let trimmed_line = line.trim();
            
            if trimmed_line.starts_with("conference_id") {
                convo_id = &trimmed_line[16..];
                convo_id = convo_id.trim();
                debug!("Got conference_id {}", convo_id);
            } else if trimmed_line.starts_with("sdp") {
                debug!("Got size {}", parsed_len);
                raw_sdp = &request[parsed_len..];
                raw_sdp = raw_sdp.trim();
                raw_sdp = &raw_sdp[4..];
                break;
            }

            parsed_len += line.len();
        }
        
        // 2. Parse the SDP into a proper structure

        let sdp:SessionDescription = SessionDescription::new();
        let parse_result:ParseResult = sdp.from_sdp(&raw_sdp);
        println!("Parsed SDP: {:?}", parse_result);

        // Check validity of the parsing
        if parse_result.unparsed_lines.len() != 0 {
            debug!("Unparsed lines is not empty, something funky");
            return;
        }
        if parse_result.ignored_lines.len() != 0 {
            debug!("Ignored lines is not empty, something funky");
            return;
        }

        // 3. Pass the SDP into a new conference OR
        //    negotiate with the current SDP that's bound
        //    to the existing conference.

        // Create new convo or return an alrady existing one
        let convo = self.convos.new_convo(convo_id);
        
        // Abstract the SDP around a member
        let member = Member::new(parse_result.desc); 

        {
            let sdp_answer;
            debug!("convo set up...");
            //let mut convo_lock = convo.write().unwrap();
            debug!("convo set up2...");
            // Add member / SDP to the convo, negotiating the SDPs
            sdp_answer = convo.add_member(member);

            // Return any response errors, like the negotiating
            // between the SDPs failing, or because the parse
            // failed. 

            debug!("SDP Answer {}", sdp_answer.clone().unwrap().to_string());
            match stream.write(sdp_answer.clone().unwrap().to_string().as_bytes()) {
                Err(x) => {
                    error!("Problem occurred writing answer {}", x);
                },
                _ => {}
            }
        }
    }
}

impl Handlers for Tcp {

    fn start_server(convos: Conferences) {
        let listener = TcpListener::bind("127.0.0.1:30000").unwrap();
        // accept connections and process them, spawning a new thread for each one
        let handler = Arc::new(Tcp {
            convos: convos,
        });

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = handler.clone();
                    thread::spawn(move || {
                        // connection succeeded
                        handler.handle_client(stream)
                    });
                }
                Err(_) => { /* connection failed */ }
            }
        }

        // close the socket server
        drop(listener);
    }
}

