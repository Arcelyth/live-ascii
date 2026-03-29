use crossterm::{
    QueueableCommand, cursor, queue,
    style::{Color, PrintStyledContent, Stylize},
    terminal,
};
use std::error::Error;
use std::io::stdout;

pub struct Context {
    pub width: u16,
    pub height: u16,
    // RGB
    pub frame_buffer: Vec<(char, (u8, u8, u8))>,
    pub image: bool,
}

impl Context {
    pub fn new(image: bool) -> Self {
        Self {
            width: 0,
            height: 0,
            frame_buffer: vec![],
            image,
        }
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, ch: char, color: (u8, u8, u8)) {
        if x < self.width && y < self.height {
            let idx = x + y * self.width;
            self.frame_buffer[idx as usize] = (ch, color);
        }
    }

    pub fn flush(&self, color: bool) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();

        if !self.image {
            stdout.queue(cursor::MoveTo(0, 0))?;
        }

        match color {
            false => {
                let frame: String = self.frame_buffer.iter().map(|pixel| pixel.0).collect();
                println!("{}", frame);
            }
            true => {
                for pixel in &self.frame_buffer {
                    let styled = pixel
                        .0
                        .with(Color::Rgb {
                            r: (pixel.1).0,
                            g: (pixel.1).1,
                            b: (pixel.1).2,
                        })
                        .on(Color::Rgb {
                            r: 10,
                            g: 10,
                            b: 10,
                        });
                    queue!(stdout, PrintStyledContent(styled))?;
                }
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let (tw, th) = terminal::size()?;
        if self.width != tw || self.height != th {
            self.width = tw;
            self.height = th;
            self.frame_buffer
                .resize((tw * th) as usize, (' ', (0, 0, 0)));
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.frame_buffer.fill((' ', (0, 0, 0)));
    }
}
