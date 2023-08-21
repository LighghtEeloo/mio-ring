use mio_core::Mio;
use std::{io::Read, net::TcpListener, path::PathBuf};

pub struct Server {
    listener: TcpListener,
    mio: Mio,
}
pub struct Client;

impl Server {
    pub fn port_path() -> PathBuf {
        mio_core::MioDirs::new().config_dir.join("port")
    }
    pub fn new() -> anyhow::Result<Self> {
        let port_path = Self::port_path();
        if port_path.exists() {
            anyhow::bail!("Server already exists");
        }
        let addr = ("127.0.0.1", 0);
        let listener = TcpListener::bind(addr)?;

        let port = listener.local_addr()?.port();
        if port_path.exists() {
            anyhow::bail!("Another server exists after bind");
        }
        std::fs::write(port_path, port.to_string())?;

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

impl Drop for Server {
    fn drop(&mut self) {
        let port_path = Self::port_path();
        if port_path.exists() {
            self.mio.flush().expect("Flush Mio");
            std::fs::remove_file(port_path).unwrap();
        } else {
            panic!("Server port file not exists");
        }
    }
}
