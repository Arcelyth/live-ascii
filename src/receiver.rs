use std::sync::mpsc::Sender;
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

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Msg {
    pub content: String,
    pub max_width: usize,
    pub max_height: usize,
    pub duration: f64,
    pub color: (u8, u8, u8),
    pub show: bool,
    pub id: String,
}

#[derive(Debug)]
pub struct MsgReceiver {
    is_running: Arc<AtomicBool>,
    port: usize,
    sender: Sender<Msg>,
}

impl MsgReceiver {
    pub fn new(port: usize, sender: Sender<Msg>) -> Self {
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

        let socket = UdpSocket::bind(format!("127.0.0.1:{}", self.port))?;
        socket.set_read_timeout(Some(Duration::from_millis(200)))?;
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let tx = self.sender.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 2048];
            while is_running.load(Ordering::SeqCst) {
                if let Ok((amt, _)) = socket.recv_from(&mut buf) {
                    if let Ok(content) = std::str::from_utf8(&buf[..amt]) {
                        let msg: Msg = if let Ok(msg) = serde_json::from_str(content) {
                            msg
                        } else {
                            let content = String::from("Failed to parse message.");
                            Msg {
                                content: content,
                                max_width: 20,
                                max_height: 5,
                                duration: 3.,
                                color: (255, 0, 0),
                                show: true,
                                id: String::new(),
                            }
                        };
                        let _ = tx.send(msg);
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
