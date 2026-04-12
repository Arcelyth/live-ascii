use std::{
    error::Error,
    io::Cursor,
    net::UdpSocket,
    sync::{Arc, Mutex},
    thread,
};

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Clone, Debug)]
pub struct Packet {
    pub now: f64,
    pub id: i32,
    pub width: f32,
    pub height: f32,
    pub eye_blink_right: f32,
    pub eye_blink_left: f32,
    pub success: u8,
    pub pnp_error: f32,
    pub quaternion: [f32; 4],
    pub euler: [f32; 3],
    pub translation: [f32; 3],
    pub lms_confidence: [f32; 68],
    pub lms: [[f32; 2]; 68],
    pub pnp_points: [[f32; 3]; 70],

    pub eye_left: f32,
    pub eye_right: f32,
    pub eye_steepness_left: f32,
    pub eye_up_down_left: f32,
    pub eye_quirk_left: f32,
    pub eye_steepness_right: f32,
    pub eye_up_down_right: f32,
    pub eye_quirk_right: f32,
    pub mouth_corner_updown_left: f32,
    pub mouth_corner_inout_left: f32,
    pub mouth_corner_updown_right: f32,
    pub mouth_corner_inout_right: f32,
    pub mouth_open: f32,
    pub mouth_wide: f32,
}

fn read_f32_array<const N: usize>(cur: &mut Cursor<&[u8]>) -> Option<[f32; N]> {
    let mut arr = [0.0f32; N];
    for v in &mut arr {
        *v = cur.read_f32::<LittleEndian>().ok()?;
    }
    Some(arr)
}

fn parse_packet(buf: &[u8]) -> Option<Packet> {
    let mut cur = Cursor::new(buf);

    let now = cur.read_f64::<LittleEndian>().ok()?;
    let id = cur.read_i32::<LittleEndian>().ok()?;
    let width = cur.read_f32::<LittleEndian>().ok()?;
    let height = cur.read_f32::<LittleEndian>().ok()?;

    let eye_blink_right = cur.read_f32::<LittleEndian>().ok()?;
    let eye_blink_left = cur.read_f32::<LittleEndian>().ok()?;

    let success = cur.read_u8().ok()?;

    let pnp_error = cur.read_f32::<LittleEndian>().ok()?;
    let quaternion = read_f32_array::<4>(&mut cur)?;
    let euler = read_f32_array::<3>(&mut cur)?;
    let translation = read_f32_array::<3>(&mut cur)?;
    let lms_confidence = read_f32_array::<68>(&mut cur)?;

    let mut lms = [[0.0f32; 2]; 68];
    for i in 0..68 {
        lms[i][1] = cur.read_f32::<LittleEndian>().ok()?;
        lms[i][0] = cur.read_f32::<LittleEndian>().ok()?;
    }

    let mut pnp_points = [[0.0f32; 3]; 70];
    for i in 0..70 {
        pnp_points[i][0] = cur.read_f32::<LittleEndian>().ok()?;
        pnp_points[i][1] = cur.read_f32::<LittleEndian>().ok()?;
        pnp_points[i][2] = cur.read_f32::<LittleEndian>().ok()?;
    }

    let eye_left = cur.read_f32::<LittleEndian>().ok()?;
    let eye_right = cur.read_f32::<LittleEndian>().ok()?;
    let eye_steepness_left = cur.read_f32::<LittleEndian>().ok()?;
    let eye_up_down_left = cur.read_f32::<LittleEndian>().ok()?;
    let eye_quirk_left = cur.read_f32::<LittleEndian>().ok()?;
    let eye_steepness_right = cur.read_f32::<LittleEndian>().ok()?;
    let eye_up_down_right = cur.read_f32::<LittleEndian>().ok()?;
    let eye_quirk_right = cur.read_f32::<LittleEndian>().ok()?;
    let mouth_corner_updown_left = cur.read_f32::<LittleEndian>().ok()?;
    let mouth_corner_inout_left = cur.read_f32::<LittleEndian>().ok()?;
    let mouth_corner_updown_right = cur.read_f32::<LittleEndian>().ok()?;
    let mouth_corner_inout_right = cur.read_f32::<LittleEndian>().ok()?;
    let mouth_open = cur.read_f32::<LittleEndian>().ok()?;
    let mouth_wide = cur.read_f32::<LittleEndian>().ok()?;

    Some(Packet {
        now,
        id,
        width,
        height,
        eye_blink_right,
        eye_blink_left,
        success,
        pnp_error,
        quaternion,
        euler,
        translation,
        lms_confidence,
        lms,
        pnp_points,
        eye_left,
        eye_right,
        eye_steepness_left,
        eye_up_down_left,
        eye_quirk_left,
        eye_steepness_right,
        eye_up_down_right,
        eye_quirk_right,
        mouth_corner_updown_left,
        mouth_corner_inout_left,
        mouth_corner_updown_right,
        mouth_corner_inout_right,
        mouth_open,
        mouth_wide,
    })
}

#[derive(Debug)]
pub struct Tracker {
    latest: Arc<Mutex<Option<Packet>>>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            latest: Arc::new(Mutex::new(None)),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind("127.0.0.1:11573")?;
        let latest_thread = Arc::clone(&self.latest);

        thread::spawn(move || {
            let mut buf = [0u8; 2048];

            loop {
                match socket.recv_from(&mut buf) {
                    Ok((amt, _src)) => {
                        if let Some(packet) = parse_packet(&buf[..amt]) {
                            if let Ok(mut g) = latest_thread.lock() {
                                *g = Some(packet);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(())
    }

    pub fn latest(&self) -> Option<Packet> {
        self.latest.lock().ok().and_then(|g| g.clone())
    }
}

