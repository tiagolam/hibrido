pub mod tcpserver;
pub mod httpserver;

pub trait handlers { 
    fn start_server(); 
}

