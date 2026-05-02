use std::{
    error::Error,
    net::UdpSocket,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct MsgReceiver {
    is_running: Arc<AtomicBool>,
    port: usize,
    sender: Sender<String>,
}

impl MsgReceiver {
    pub fn new(port: usize, sender: Sender<String>) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            port,
            sender,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let tx = self.sender.clone();
        let port = self.port;

        thread::spawn(move || {
            let socket = UdpSocket::bind(format!("127.0.0.1:{}", port)).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();

            let mut buf = [0u8; 2048];
            while is_running.load(Ordering::SeqCst) {
                if let Ok((amt, _)) = socket.recv_from(&mut buf) {
                    if let Ok(content) = std::str::from_utf8(&buf[..amt]) {
                        let _ = tx.send(content.to_string());
                    }
                }
            }
        });
        Ok(())
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
