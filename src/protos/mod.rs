pub mod tcpserver;
pub mod httpserver;

pub trait Handlers { 
    fn start_server(); 
}

