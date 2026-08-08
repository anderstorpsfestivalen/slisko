//! Persistent graphical DDP monitor for slisko.
//!
//! The window and UDP listener are independent of the pattern runner. A host
//! runner can therefore be stopped, rebuilt, and restarted while this process
//! keeps displaying the last complete frame.

use std::collections::HashMap;
use std::fmt;
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;
use ddp_rs::packet::PacketRef;
use ddp_rs::protocol::{Header, ID, PixelConfig};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Image, Mesh, Rect};
use ggez::{Context, ContextBuilder, GameError, GameResult};
use slisko_core::chassi::Chassi;
use slisko_core::output::StrandMap;

const CARD_PITCH: f32 = 108.0;
const CHASSIS_HEIGHT: f32 = 1000.0;
const RECEIVE_TIMEOUT: Duration = Duration::from_millis(250);
const PARTIAL_FRAME_TTL: Duration = Duration::from_secs(2);

#[derive(Debug, Parser)]
#[command(about = "Keep a chassis window open and display incoming DDP frames")]
pub struct Args {
    /// UDP address on which to receive DDP frames.
    #[arg(long, default_value = "0.0.0.0:4048")]
    pub listen: String,

    /// Directory containing the linecard PNG files.
    #[arg(long)]
    pub assets: Option<PathBuf>,
}

fn default_assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/images")
}

#[derive(Clone, Debug)]
struct Assembly {
    rgb: Vec<u8>,
    received: Vec<bool>,
    saw_push: bool,
    last_update: Instant,
}

impl Assembly {
    fn new(frame_len: usize, now: Instant) -> Self {
        Self {
            rgb: vec![0; frame_len],
            received: vec![false; frame_len],
            saw_push: false,
            last_update: now,
        }
    }

    fn reset(&mut self, now: Instant) {
        self.rgb.fill(0);
        self.received.fill(false);
        self.saw_push = false;
        self.last_update = now;
    }

    fn complete(&self) -> bool {
        self.saw_push && self.received.iter().all(|received| *received)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PacketError {
    UnsupportedPacket,
    Truncated { declared: usize, actual: usize },
    OutOfRange { offset: usize, length: usize },
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PacketError::UnsupportedPacket => write!(f, "not an RGB8 pixel-data packet"),
            PacketError::Truncated { declared, actual } => write!(
                f,
                "packet declares {declared} payload bytes but carries {actual}"
            ),
            PacketError::OutOfRange { offset, length } => {
                write!(
                    f,
                    "packet range {offset}..{} is outside the strand",
                    offset + length
                )
            }
        }
    }
}

#[derive(Debug)]
struct DdpAssembler {
    frame_len: usize,
    sources: HashMap<SocketAddr, Assembly>,
}

impl DdpAssembler {
    fn new(frame_len: usize) -> Self {
        Self {
            frame_len,
            sources: HashMap::new(),
        }
    }

    fn ingest(
        &mut self,
        source: SocketAddr,
        header: Header,
        data: &[u8],
        now: Instant,
    ) -> Result<Option<Vec<u8>>, PacketError> {
        if header.pixel_config != PixelConfig::default()
            || !matches!(header.id, ID::Default | ID::Broadcast)
            || header.packet_type.query
            || header.packet_type.reply
        {
            return Err(PacketError::UnsupportedPacket);
        }

        let length = header.length as usize;
        if data.len() < length {
            return Err(PacketError::Truncated {
                declared: length,
                actual: data.len(),
            });
        }
        let offset = header.offset as usize;
        let Some(end) = offset.checked_add(length) else {
            return Err(PacketError::OutOfRange { offset, length });
        };
        if end > self.frame_len {
            return Err(PacketError::OutOfRange { offset, length });
        }

        let assembly = self
            .sources
            .entry(source)
            .or_insert_with(|| Assembly::new(self.frame_len, now));
        // Every full frame emitted by ddp-rs starts at byte offset zero. This
        // deliberately ignores sequence continuity, so restarted senders and
        // their reset sequence numbers are accepted immediately.
        if offset == 0 {
            assembly.reset(now);
        }
        assembly.rgb[offset..end].copy_from_slice(&data[..length]);
        assembly.received[offset..end].fill(true);
        assembly.saw_push |= header.packet_type.push;
        assembly.last_update = now;

        if assembly.complete() {
            let complete = assembly.rgb.clone();
            assembly.reset(now);
            Ok(Some(complete))
        } else {
            Ok(None)
        }
    }

    fn expire(&mut self, now: Instant) {
        self.sources
            .retain(|_, assembly| now.duration_since(assembly.last_update) < PARTIAL_FRAME_TTL);
    }
}

type LatestFrame = Arc<Mutex<Option<Vec<u8>>>>;

fn spawn_receiver(listen: String, latest: LatestFrame) {
    thread::Builder::new()
        .name("ddp-receiver".into())
        .spawn(move || receiver_loop(&listen, latest))
        .expect("spawn DDP receiver");
}

fn receiver_loop(listen: &str, latest: LatestFrame) {
    loop {
        let socket = match UdpSocket::bind(listen) {
            Ok(socket) => socket,
            Err(error) => {
                eprintln!("slisko-sim: cannot bind {listen}: {error}; retrying");
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
        if let Err(error) = socket.set_read_timeout(Some(RECEIVE_TIMEOUT)) {
            eprintln!("slisko-sim: cannot configure DDP socket: {error}");
        }
        eprintln!("slisko-sim: listening for DDP on {listen}");

        let mut assembler = DdpAssembler::new(slisko_config::LED_COUNT * 3);
        let mut packet_buf = [0u8; 2048];
        loop {
            match socket.recv_from(&mut packet_buf) {
                Ok((size, source)) => {
                    let Some(packet) = PacketRef::from_bytes(&packet_buf[..size]) else {
                        eprintln!("slisko-sim: ignored malformed DDP packet from {source}");
                        continue;
                    };
                    match assembler.ingest(source, packet.header, packet.data, Instant::now()) {
                        Ok(Some(frame)) => {
                            *latest.lock().unwrap() = Some(frame);
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!("slisko-sim: ignored DDP packet from {source}: {error}")
                        }
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    assembler.expire(Instant::now());
                }
                Err(error) => {
                    eprintln!("slisko-sim: DDP receive failed: {error}; rebinding");
                    break;
                }
            }
        }
    }
}

struct CardImage {
    x: f32,
    image: Image,
}

struct Simulator {
    cards: Vec<CardImage>,
    led_circle: Mesh,
    chassi: Chassi,
    strand_map: StrandMap,
    latest: LatestFrame,
    width: f32,
}

impl Simulator {
    fn new(ctx: &mut Context, latest: LatestFrame) -> GameResult<Self> {
        let chassi = Chassi::from_specs(slisko_config::CHASSIS);
        let strand_map = StrandMap::new(
            &chassi,
            slisko_config::OUTPUT_MAPPING,
            slisko_config::LED_COUNT,
        )
        .map_err(|error| GameError::ConfigError(error.to_string()))?;
        let width = CARD_PITCH * chassi.linecards.len() as f32;

        let mut cards = Vec::with_capacity(chassi.linecards.len());
        for (index, card) in chassi.linecards.iter().enumerate() {
            let path = format!("/{}", card.image);
            let image = Image::from_path(ctx, path)?;
            // The Go simulator centers every sprite on a 108-pixel card slot.
            let x = index as f32 * CARD_PITCH + (CARD_PITCH - image.width() as f32) / 2.0;
            cards.push(CardImage { x, image });
        }

        let led_circle =
            Mesh::new_circle(ctx, DrawMode::fill(), Vec2::ZERO, 1.0, 0.1, Color::WHITE)?;

        Ok(Self {
            cards,
            led_circle,
            chassi,
            strand_map,
            latest,
            width,
        })
    }
}

impl EventHandler for Simulator {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if let Some(frame) = self.latest.lock().unwrap().take() {
            self.strand_map
                .apply_rgb(&frame, &mut self.chassi.leds)
                .map_err(|error| GameError::RenderError(error.to_string()))?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        canvas.set_screen_coordinates(Rect::new(0.0, 0.0, self.width, CHASSIS_HEIGHT));

        for card in &self.cards {
            canvas.draw(
                &card.image,
                DrawParam::default().dest(Vec2::new(card.x, 0.0)),
            );
        }

        for (card_index, card) in self.chassi.linecards.iter().enumerate() {
            let start = card.led_offset;
            let end = start + card.led_count;
            for led in &self.chassi.leds[start..end] {
                // The Go renderer converts its bottom-left world coordinates
                // back to image coordinates; ggez is already top-left based.
                let position = Vec2::new(led.pos.x + CARD_PITCH * card_index as f32, led.pos.y);
                canvas.draw(
                    &self.led_circle,
                    DrawParam::default()
                        .dest(position)
                        .scale(Vec2::splat(led.pos.size))
                        .color(Color::new(led.r, led.g, led.b, 1.0)),
                );
            }
        }

        canvas.finish(ctx)
    }
}

pub fn run() -> GameResult {
    let args = Args::parse();
    let assets = args.assets.unwrap_or_else(default_assets);
    let width = CARD_PITCH * slisko_config::CHASSIS.len() as f32;

    let (mut ctx, event_loop) = ContextBuilder::new("slisko-sim", "anderstorpsfestivalen")
        .add_resource_path(&assets)
        .window_setup(WindowSetup::default().title("Slisko Simulator").vsync(true))
        .window_mode(
            WindowMode::default()
                .dimensions(width, CHASSIS_HEIGHT)
                .resizable(true),
        )
        .build()?;

    let latest = Arc::new(Mutex::new(None));
    spawn_receiver(args.listen, latest.clone());
    let simulator = Simulator::new(&mut ctx, latest)?;
    event::run(ctx, event_loop, simulator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddp_rs::protocol::PacketType;

    fn source(port: u16) -> SocketAddr {
        format!("127.0.0.1:{port}").parse().unwrap()
    }

    fn header(offset: usize, length: usize, push: bool, sequence: u8) -> Header {
        Header {
            packet_type: PacketType {
                push,
                ..Default::default()
            },
            sequence_number: sequence,
            pixel_config: PixelConfig::default(),
            id: ID::Default,
            offset: offset as u32,
            length: length as u16,
            ..Default::default()
        }
    }

    #[test]
    fn assembles_chunked_frames_and_waits_for_push() {
        let now = Instant::now();
        let mut assembler = DdpAssembler::new(9);
        assert_eq!(
            assembler
                .ingest(source(1), header(0, 6, false, 1), &[1, 2, 3, 4, 5, 6], now)
                .unwrap(),
            None
        );
        assert_eq!(
            assembler
                .ingest(source(1), header(6, 3, true, 2), &[7, 8, 9], now)
                .unwrap(),
            Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 9])
        );
    }

    #[test]
    fn accepts_reordered_chunks_after_frame_start() {
        let now = Instant::now();
        let mut assembler = DdpAssembler::new(9);
        assembler
            .ingest(source(1), header(0, 3, false, 1), &[1, 2, 3], now)
            .unwrap();
        assembler
            .ingest(source(1), header(6, 3, true, 3), &[7, 8, 9], now)
            .unwrap();
        let frame = assembler
            .ingest(source(1), header(3, 3, false, 2), &[4, 5, 6], now)
            .unwrap();
        assert_eq!(frame, Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]));
    }

    #[test]
    fn sender_restart_with_new_port_and_reset_sequence_is_accepted() {
        let now = Instant::now();
        let mut assembler = DdpAssembler::new(3);
        let first = assembler
            .ingest(source(1), header(0, 3, true, 12), &[1, 2, 3], now)
            .unwrap();
        let restarted = assembler
            .ingest(source(2), header(0, 3, true, 1), &[4, 5, 6], now)
            .unwrap();
        assert_eq!(first, Some(vec![1, 2, 3]));
        assert_eq!(restarted, Some(vec![4, 5, 6]));
    }

    #[test]
    fn rejects_truncated_and_out_of_range_packets() {
        let now = Instant::now();
        let mut assembler = DdpAssembler::new(3);
        assert_eq!(
            assembler.ingest(source(1), header(0, 3, true, 1), &[1], now),
            Err(PacketError::Truncated {
                declared: 3,
                actual: 1
            })
        );
        assert_eq!(
            assembler.ingest(source(1), header(2, 3, true, 1), &[1, 2, 3], now),
            Err(PacketError::OutOfRange {
                offset: 2,
                length: 3
            })
        );
    }

    #[test]
    fn expires_abandoned_source_assemblies() {
        let now = Instant::now();
        let mut assembler = DdpAssembler::new(6);
        assembler
            .ingest(source(1), header(0, 3, false, 1), &[1, 2, 3], now)
            .unwrap();
        assembler.expire(now + PARTIAL_FRAME_TTL + Duration::from_millis(1));
        assert!(assembler.sources.is_empty());
    }
}
