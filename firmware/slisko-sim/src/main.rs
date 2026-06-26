//! slisko desktop simulator (host).
//!
//! Drives the `slisko-core` `Controller` over a small chassis and prints frames
//! as ANSI color blocks — same engine the firmware runs. A windowed backend over
//! the baked X/Y positions can replace the ANSI dump later.

use slisko_core::chassi::{Chassi, LineCardSpec};
use slisko_core::controller::Controller;
use slisko_core::pixel::Position;
use slisko_core::traffic::Shaper;

// A throwaway 24-pixel single-card chassis until the baker emits real specs.
static POS: [Position; 24] = [Position {
    x: 0.0,
    y: 0.0,
    size: 1.0,
}; 24];

fn main() {
    let specs = [LineCardSpec {
        name: "SIM",
        image: "",
        active: true,
        positions: &POS,
        link: &[],
        status: None,
        labeled: &[],
    }];
    let chassi = Chassi::from_specs(&specs);

    let mut ctrl = Controller::new(chassi, Shaper::default(), 0xC0FFEE);
    ctrl.enable("colorcycler");

    let fps = 10;
    for frame in 0..30 {
        let secs = frame as f32 / fps as f32;
        ctrl.tick(secs);
        print_frame(ctrl.leds());
    }
    println!();
}

fn print_frame(leds: &[slisko_core::pixel::Pixel]) {
    let mut line = String::new();
    for px in leds {
        let [r, g, b] = px.to_rgb8();
        line.push_str(&format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m"));
    }
    println!("{line}");
}
