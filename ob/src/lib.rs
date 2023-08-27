use mio_core::Mio;
use std::{io::Read, net::TcpListener};

pub struct Server {
    listener: TcpListener,
    mio: Mio,
}
pub struct Client;

impl Server {
    pub fn port() -> u16 {
        9720
    }
    pub fn new() -> anyhow::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", Self::port()))?;

        let mio = Mio::read_or_bak_with_default();
        Ok(Self { listener, mio })
    }
    pub fn run(&self) -> anyhow::Result<()> {
        loop {
            let (mut stream, addr) = self.listener.accept()?;
            let mut buffer = [0; 1024];
            stream.read(&mut buffer)?;
        }
    }
}
