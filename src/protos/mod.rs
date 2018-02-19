pub mod tcpserver;
pub mod httpserver;
use convo::convo::{Conferences};

pub trait Handlers { 
    fn start_server(convos: Conferences);
}

