//! Native host harness for the portable slisko render engine.

use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Local, Timelike};
use clap::Parser;
use ddp_rs::connection::DDPConnection;
use ddp_rs::protocol::{ID, PixelConfig};
use slisko_core::chassi::Chassi;
use slisko_core::controller::{Controller, PATTERN_NAMES};
use slisko_core::output::StrandMap;
use slisko_core::traffic::Shaper;

const DEFAULT_SEED: u64 = 0xC0FFEE;

#[derive(Debug, Parser)]
#[command(about = "Run slisko patterns natively and stream the mapped strand over DDP")]
pub struct Args {
    /// DDP destination as host:port.
    #[arg(long, default_value = "127.0.0.1:4048")]
    pub ddp: String,

    /// Render and transmit frames per second.
    #[arg(long, default_value_t = 60, value_parser = clap::value_parser!(u32).range(1..=1000))]
    pub fps: u32,

    /// Deterministic pattern RNG seed.
    #[arg(long, default_value_t = DEFAULT_SEED)]
    pub seed: u64,

    /// Fixed fractional hour-of-day for the traffic shaper; local time is used when omitted.
    #[arg(long, value_parser = parse_hour)]
    pub hour: Option<f32>,

    /// Pattern to enable. Supplying any --pattern replaces the baked defaults.
    #[arg(long = "pattern")]
    pub patterns: Vec<String>,
}

fn parse_hour(value: &str) -> Result<f32, String> {
    let hour = value
        .parse::<f32>()
        .map_err(|_| format!("{value:?} is not a number"))?;
    if (0.0..24.0).contains(&hour) {
        Ok(hour)
    } else {
        Err("hour must be in the range 0.0..24.0".into())
    }
}

fn local_hour() -> f32 {
    let now = Local::now();
    now.hour() as f32 + now.minute() as f32 / 60.0 + now.second() as f32 / 3600.0
}

pub fn run() {
    if let Err(error) = run_with(Args::parse()) {
        eprintln!("slisko-host: {error}");
        std::process::exit(1);
    }
}

fn run_with(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let selected_patterns: Vec<&str> = if args.patterns.is_empty() {
        slisko_config::ACTIVE_PATTERNS.to_vec()
    } else {
        for name in &args.patterns {
            if !PATTERN_NAMES.contains(&name.as_str()) {
                return Err(format!(
                    "unknown pattern {name:?}; available patterns: {}",
                    PATTERN_NAMES.join(", ")
                )
                .into());
            }
        }
        args.patterns.iter().map(String::as_str).collect()
    };

    let chassi = Chassi::from_specs(slisko_config::CHASSIS);
    let strand_map = StrandMap::new(
        &chassi,
        slisko_config::OUTPUT_MAPPING,
        slisko_config::LED_COUNT,
    )
    .map_err(|error| format!("invalid baked output mapping: {error}"))?;
    let mut controller = Controller::new(chassi, Shaper::new(slisko_config::SHAPER), args.seed);
    controller.set_hour(args.hour.unwrap_or_else(local_hour));
    for pattern in &selected_patterns {
        controller.enable(pattern);
    }

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let mut ddp = DDPConnection::try_new(&args.ddp, PixelConfig::default(), ID::Default, socket)?;
    let frame_time = Duration::from_secs_f64(1.0 / args.fps as f64);
    let start = Instant::now();
    let mut next_frame = start;
    let mut rgb = Vec::with_capacity(strand_map.len() * 3);

    eprintln!(
        "slisko-host: streaming {} LEDs at {} fps to {} with [{}]",
        strand_map.len(),
        args.fps,
        args.ddp,
        selected_patterns.join(", ")
    );

    loop {
        let now = Instant::now();
        if now < next_frame {
            thread::sleep(next_frame - now);
        }
        let elapsed = start.elapsed().as_secs_f32();
        if args.hour.is_none() && controller.frame() % args.fps as i64 == 0 {
            controller.set_hour(local_hour());
        }
        controller.tick(elapsed);
        strand_map.encode_rgb(controller.leds(), &mut rgb);
        ddp.write(&rgb)?;

        next_frame += frame_time;
        let after_send = Instant::now();
        if next_frame < after_send {
            next_frame = after_send;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddp_rs::packet::PacketRef;

    fn receive_frame(socket: &UdpSocket, frame_len: usize) -> Vec<u8> {
        let mut frame = vec![0; frame_len];
        let mut received = vec![false; frame_len];
        let mut packet_buf = [0u8; 2048];
        loop {
            let (size, _) = socket.recv_from(&mut packet_buf).unwrap();
            let packet = PacketRef::from_bytes(&packet_buf[..size]).unwrap();
            let offset = packet.header.offset as usize;
            let length = packet.header.length as usize;
            frame[offset..offset + length].copy_from_slice(&packet.data[..length]);
            received[offset..offset + length].fill(true);
            if packet.header.packet_type.push && received.iter().all(|value| *value) {
                return frame;
            }
        }
    }

    #[test]
    fn hour_parser_enforces_day_range() {
        assert_eq!(parse_hour("12.5").unwrap(), 12.5);
        assert!(parse_hour("24").is_err());
        assert!(parse_hour("-1").is_err());
        assert!(parse_hour("noon").is_err());
    }

    #[test]
    fn every_pattern_renders_deterministically_on_the_baked_chassis() {
        for &name in PATTERN_NAMES {
            let make_controller = || {
                let mut controller = Controller::new(
                    Chassi::from_specs(slisko_config::CHASSIS),
                    Shaper::new(slisko_config::SHAPER),
                    DEFAULT_SEED,
                );
                controller.set_hour(12.0);
                controller.enable(name);
                controller
            };
            let mut a = make_controller();
            let mut b = make_controller();
            for frame in 0..120 {
                let now = frame as f32 / 60.0;
                a.tick(now);
                b.tick(now);
            }
            assert_eq!(a.leds(), b.leds(), "pattern {name} was not deterministic");
            assert!(
                a.leds().iter().all(|pixel| {
                    pixel.r.is_finite()
                        && pixel.g.is_finite()
                        && pixel.b.is_finite()
                        && (0.0..=1.0).contains(&pixel.r)
                        && (0.0..=1.0).contains(&pixel.g)
                        && (0.0..=1.0).contains(&pixel.b)
                }),
                "pattern {name} emitted an invalid color"
            );
        }
    }

    #[test]
    fn ddp_loopback_chunks_frames_and_accepts_a_restarted_sender() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let destination = receiver.local_addr().unwrap().to_string();
        let first = (0..2000).map(|i| (i % 251) as u8).collect::<Vec<_>>();

        let mut sender = DDPConnection::try_new(
            &destination,
            PixelConfig::default(),
            ID::Default,
            UdpSocket::bind("127.0.0.1:0").unwrap(),
        )
        .unwrap();
        sender.write(&first).unwrap();
        assert_eq!(receive_frame(&receiver, first.len()), first);

        drop(sender);
        let restarted = vec![42; 2000];
        let mut sender = DDPConnection::try_new(
            &destination,
            PixelConfig::default(),
            ID::Default,
            UdpSocket::bind("127.0.0.1:0").unwrap(),
        )
        .unwrap();
        sender.write(&restarted).unwrap();
        assert_eq!(receive_frame(&receiver, restarted.len()), restarted);
    }
}
