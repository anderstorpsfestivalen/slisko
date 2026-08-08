//! Host-side compiler for slisko's TOML hardware configurations.
//!
//! This crate owns the line-card catalog that previously lived in the Go
//! `pkg/chassi` package. `config` calls it from `build.rs`, while the
//! binary exposes useful validation and source-rendering commands. Generated
//! Rust is built as `quote` tokens, parsed as a `syn::File`, and formatted with
//! `prettyplease`.

use engine::controller::PATTERN_NAMES;
use engine::output::LedType;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use syn::LitStr;

#[derive(Debug)]
pub struct BakeError {
    message: String,
}

impl BakeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for BakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BakeError {}

pub type Result<T> = std::result::Result<T, BakeError>;

/// A validated, expanded configuration ready to render as Rust source.
pub struct BakedConfig {
    source: PathBuf,
    led_count: usize,
    cards: Vec<CardDefinition>,
    mapping: Vec<Mapping>,
    patterns: Vec<String>,
    led_type: String,
    led_outputs: Vec<LedOutput>,
    shaper: TrafficShaper,
    buttons: Vec<Button>,
}

impl BakedConfig {
    pub fn led_count(&self) -> usize {
        self.led_count
    }

    pub fn card_count(&self) -> usize {
        self.cards.len()
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Render the static module consumed by `config`.
    pub fn render(&self) -> String {
        let card_tables = self
            .cards
            .iter()
            .enumerate()
            .map(|(index, card)| {
                let positions_ident = format_ident!("LC{index}_POS");
                let links_ident = format_ident!("LC{index}_LINK");
                let labels_ident = format_ident!("LC{index}_LABELED");
                let positions = card.positions.iter().map(|position| {
                    let x = Literal::f32_unsuffixed(position.x);
                    let y = Literal::f32_unsuffixed(position.y);
                    let size = Literal::f32_unsuffixed(position.size);
                    quote!(Position { x: #x, y: #y, size: #size })
                });
                let links = card
                    .link
                    .iter()
                    .map(|&index| Literal::usize_unsuffixed(index));
                let labeled = card.labeled.iter().map(|(label, index)| {
                    let label = LitStr::new(label, Span::call_site());
                    let index = Literal::usize_unsuffixed(*index);
                    quote!((#label, #index))
                });

                quote! {
                    static #positions_ident: &[Position] = &[#(#positions),*];
                    static #links_ident: &[usize] = &[#(#links),*];
                    static #labels_ident: &[(&str, usize)] = &[#(#labeled),*];
                }
            })
            .collect::<Vec<TokenStream>>();

        let chassis = self
            .cards
            .iter()
            .enumerate()
            .map(|(index, card)| {
                let positions = format_ident!("LC{index}_POS");
                let links = format_ident!("LC{index}_LINK");
                let labeled = format_ident!("LC{index}_LABELED");
                let name = LitStr::new(card.name, Span::call_site());
                let image = LitStr::new(card.image, Span::call_site());
                let active = card.active;
                let status = card.status.map_or_else(
                    || quote!(None),
                    |index| {
                        let index = Literal::usize_unsuffixed(index);
                        quote!(Some(#index))
                    },
                );
                quote! {
                    LineCardSpec {
                        name: #name,
                        image: #image,
                        active: #active,
                        positions: #positions,
                        link: #links,
                        status: #status,
                        labeled: #labeled,
                    }
                }
            })
            .collect::<Vec<TokenStream>>();

        let mapping = self
            .mapping
            .iter()
            .map(|mapping| match mapping {
                Mapping::Card(card) => {
                    let card = Literal::usize_unsuffixed(*card);
                    quote!(MappingSegment::Card(#card))
                }
                Mapping::Gap(count) => {
                    let count = Literal::usize_unsuffixed(*count);
                    quote!(MappingSegment::Gap(#count))
                }
            })
            .collect::<Vec<TokenStream>>();
        let patterns = self
            .patterns
            .iter()
            .map(|pattern| LitStr::new(pattern, Span::call_site()))
            .collect::<Vec<LitStr>>();
        let led_type = LitStr::new(&self.led_type, Span::call_site());
        let led_outputs = self.led_outputs.iter().map(|output| {
            let gpio = Literal::u8_unsuffixed(output.gpio);
            let start = Literal::usize_unsuffixed(output.start);
            let end = Literal::usize_unsuffixed(output.end);
            quote!(LedOutput { gpio: #gpio, start: #start, end: #end })
        });
        let buttons = self.buttons.iter().map(|button| {
            let gpio = Literal::u8_unsuffixed(button.gpio);
            let scene = button
                .scene
                .iter()
                .map(|pattern| LitStr::new(pattern, Span::call_site()));
            quote!(Button { gpio: #gpio, scene: &[#(#scene),*] })
        });
        let led_count = Literal::usize_unsuffixed(self.led_count);
        let enabled = self.shaper.enabled;
        let peak_start = Literal::f32_unsuffixed(self.shaper.peak_start as f32);
        let peak_end = Literal::f32_unsuffixed(self.shaper.peak_end as f32);
        let low_start = Literal::f32_unsuffixed(self.shaper.low_start as f32);
        let low_end = Literal::f32_unsuffixed(self.shaper.low_end as f32);
        let peak_factor = Literal::f32_unsuffixed(self.shaper.peak_factor);
        let low_factor = Literal::f32_unsuffixed(self.shaper.low_factor);

        let tokens = quote! {
            use crate::{Button, LedOutput};
            use engine::chassi::LineCardSpec;
            use engine::output::MappingSegment;
            use engine::pixel::Position;
            use engine::traffic::ShaperConfig;

            #(#card_tables)*

            pub static CHASSIS: &[LineCardSpec] = &[#(#chassis),*];
            pub const LED_COUNT: usize = #led_count;
            pub static OUTPUT_MAPPING: &[MappingSegment] = &[#(#mapping),*];
            pub static ACTIVE_PATTERNS: &[&str] = &[#(#patterns),*];
            pub static LED_TYPE: &str = #led_type;
            pub static LED_OUTPUTS: &[LedOutput] = &[#(#led_outputs),*];
            pub static SHAPER: ShaperConfig = ShaperConfig {
                enabled: #enabled,
                peak_start: #peak_start,
                peak_end: #peak_end,
                low_start: #low_start,
                low_end: #low_end,
                peak_factor: #peak_factor,
                low_factor: #low_factor,
            };
            pub static BUTTONS: &[Button] = &[#(#buttons),*];
        };
        let syntax: syn::File = syn::parse2(tokens).expect("generated token stream must be valid");
        let body = prettyplease::unparse(&syntax);
        let source = self
            .source
            .file_name()
            .unwrap_or(self.source.as_os_str())
            .to_string_lossy();
        format!("// @generated by baker from {source} — DO NOT EDIT.\n{body}")
    }
}

/// Read, parse, validate, and expand one configuration.
pub fn bake_path(path: impl AsRef<Path>) -> Result<BakedConfig> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)
        .map_err(|error| BakeError::new(format!("read {}: {error}", path.display())))?;
    bake_text(path, &source)
}

/// Bake one configuration directly into its generated Rust module.
pub fn bake_to_string(path: impl AsRef<Path>) -> Result<String> {
    bake_path(path).map(|baked| baked.render())
}

fn bake_text(source: &Path, input: &str) -> Result<BakedConfig> {
    let raw: RawConfig = toml::from_str(input)
        .map_err(|error| BakeError::new(format!("parse {}: {error}", source.display())))?;
    validate_and_expand(source, raw)
}

fn validate_and_expand(source: &Path, raw: RawConfig) -> Result<BakedConfig> {
    if raw.led_amount == 0 {
        return Err(BakeError::new("LEDAmount: must be greater than zero"));
    }
    if raw.linecards.is_empty() {
        return Err(BakeError::new("Linecards: must contain at least one card"));
    }

    let mut cards = Vec::with_capacity(raw.linecards.len());
    for (index, name) in raw.linecards.iter().enumerate() {
        let card = card_definition(name).ok_or_else(|| {
            BakeError::new(format!("Linecards[{index}]: unknown linecard {name:?}"))
        })?;
        card.validate_catalog(name)?;
        cards.push(card);
    }

    let mut mapped_leds = 0usize;
    let mut mapping = Vec::with_capacity(raw.mapping.len());
    for (index, entry) in raw.mapping.into_iter().enumerate() {
        let normalized = match (entry.card, entry.r#gen) {
            (Some(_), Some(_)) => {
                return Err(BakeError::new(format!(
                    "Mapping[{index}]: must not specify both 'card' and 'gen'"
                )));
            }
            (None, None) => {
                return Err(BakeError::new(format!(
                    "Mapping[{index}]: must specify either 'card' or 'gen'"
                )));
            }
            (Some(card), None) => {
                let card = nonnegative_usize(card, &format!("Mapping[{index}].card"))?;
                let definition = cards.get(card).ok_or_else(|| {
                    BakeError::new(format!(
                        "Mapping[{index}].card: index {card} is out of bounds for {} linecards",
                        cards.len()
                    ))
                })?;
                mapped_leds = mapped_leds
                    .checked_add(definition.positions.len())
                    .ok_or_else(|| BakeError::new("Mapping: expanded LED count overflowed"))?;
                Mapping::Card(card)
            }
            (None, Some(gap)) => {
                let gap = nonnegative_usize(gap, &format!("Mapping[{index}].gen"))?;
                mapped_leds = mapped_leds
                    .checked_add(gap)
                    .ok_or_else(|| BakeError::new("Mapping: expanded LED count overflowed"))?;
                Mapping::Gap(gap)
            }
        };
        if mapped_leds > raw.led_amount {
            return Err(BakeError::new(format!(
                "Mapping[{index}]: expands to {mapped_leds} LEDs, exceeding LEDAmount {}",
                raw.led_amount
            )));
        }
        mapping.push(normalized);
    }

    validate_patterns("Patterns", &raw.patterns)?;

    let (led_type, led_outputs) = match raw.led_info {
        Some(info) => {
            if info.kind.trim().is_empty() {
                return Err(BakeError::new("ledinfo.type: must not be empty"));
            }
            if LedType::parse(&info.kind).is_none() {
                return Err(BakeError::new(format!(
                    "ledinfo.type: unsupported LED type {:?}",
                    info.kind
                )));
            }
            let mut outputs = Vec::with_capacity(info.mapping.len());
            for (index, output) in info.mapping.into_iter().enumerate() {
                let field = format!("ledinfo.mapping[{index}]");
                let gpio = u8::try_from(output.gpio).map_err(|_| {
                    BakeError::new(format!("{field}.gpio: {} is outside 0..=255", output.gpio))
                })?;
                let (start, end) = parse_range(&output.range, &format!("{field}.range"))?;
                if start >= end {
                    return Err(BakeError::new(format!(
                        "{field}.range: {:?} is empty or descending",
                        output.range
                    )));
                }
                if end > raw.led_amount {
                    return Err(BakeError::new(format!(
                        "{field}.range: end {end} exceeds LEDAmount {}",
                        raw.led_amount
                    )));
                }
                outputs.push(LedOutput { gpio, start, end });
            }
            (info.kind, outputs)
        }
        None => ("WS2812".to_owned(), Vec::new()),
    };

    let shaper = raw.traffic_shaper.unwrap_or_default();
    shaper.validate()?;

    let mut buttons = Vec::with_capacity(raw.buttons.len());
    for (index, button) in raw.buttons.into_iter().enumerate() {
        let gpio = parse_gpio(&button.pin, &format!("Buttons[{index}].pin"))?;
        validate_patterns(&format!("Buttons[{index}].action"), &button.action)?;
        buttons.push(Button {
            gpio,
            scene: button.action,
        });
    }

    Ok(BakedConfig {
        source: source.to_owned(),
        led_count: raw.led_amount,
        cards,
        mapping,
        patterns: raw.patterns,
        led_type,
        led_outputs,
        shaper,
        buttons,
    })
}

fn validate_patterns(field: &str, patterns: &[String]) -> Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        if !PATTERN_NAMES.contains(&pattern.as_str()) {
            return Err(BakeError::new(format!(
                "{field}[{index}]: unknown Rust pattern {pattern:?}"
            )));
        }
    }
    Ok(())
}

fn nonnegative_usize(value: i64, field: &str) -> Result<usize> {
    usize::try_from(value)
        .map_err(|_| BakeError::new(format!("{field}: {value} must be non-negative")))
}

fn parse_range(value: &str, field: &str) -> Result<(usize, usize)> {
    let (start, end) = value
        .split_once('-')
        .ok_or_else(|| BakeError::new(format!("{field}: {value:?} must use start-end")))?;
    let start = start
        .trim()
        .parse::<usize>()
        .map_err(|error| BakeError::new(format!("{field}: invalid start in {value:?}: {error}")))?;
    let end = end
        .trim()
        .parse::<usize>()
        .map_err(|error| BakeError::new(format!("{field}: invalid end in {value:?}: {error}")))?;
    Ok((start, end))
}

fn parse_gpio(value: &str, field: &str) -> Result<u8> {
    let digits = value
        .strip_prefix("GPIO")
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .ok_or_else(|| BakeError::new(format!("{field}: {value:?} must use the form GPIO23")))?;
    digits
        .parse::<u8>()
        .map_err(|error| BakeError::new(format!("{field}: invalid GPIO in {value:?}: {error}")))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(rename = "LEDAmount")]
    led_amount: usize,
    #[serde(rename = "Linecards")]
    linecards: Vec<String>,
    #[serde(rename = "Patterns", default)]
    patterns: Vec<String>,
    #[serde(rename = "Mapping", default)]
    mapping: Vec<RawMapping>,
    #[serde(rename = "Buttons", default)]
    buttons: Vec<RawButton>,
    #[serde(default)]
    traffic_shaper: Option<TrafficShaper>,
    #[serde(default, rename = "output")]
    _output: Option<RawOutput>,
    #[serde(default, rename = "ledinfo")]
    led_info: Option<RawLedInfo>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMapping {
    card: Option<i64>,
    r#gen: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawButton {
    pin: String,
    #[serde(default)]
    action: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLedInfo {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    mapping: Vec<RawLedOutput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLedOutput {
    gpio: i64,
    range: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct RawOutput {
    ddp: Option<RawDdpOutput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct RawDdpOutput {
    host: String,
    port: u16,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrafficShaper {
    enabled: bool,
    peak_start: i32,
    peak_end: i32,
    low_start: i32,
    low_end: i32,
    peak_factor: f32,
    low_factor: f32,
}

impl TrafficShaper {
    fn validate(self) -> Result<()> {
        for (field, hour) in [
            ("traffic_shaper.peak_start", self.peak_start),
            ("traffic_shaper.peak_end", self.peak_end),
            ("traffic_shaper.low_start", self.low_start),
            ("traffic_shaper.low_end", self.low_end),
        ] {
            if !(0..=23).contains(&hour) {
                return Err(BakeError::new(format!("{field}: {hour} is outside 0..=23")));
            }
        }
        for (field, factor) in [
            ("traffic_shaper.peak_factor", self.peak_factor),
            ("traffic_shaper.low_factor", self.low_factor),
        ] {
            if !factor.is_finite() || factor < 0.0 {
                return Err(BakeError::new(format!(
                    "{field}: {factor} must be finite and non-negative"
                )));
            }
        }
        Ok(())
    }
}

impl Default for TrafficShaper {
    fn default() -> Self {
        Self {
            enabled: true,
            peak_start: 17,
            peak_end: 22,
            low_start: 2,
            low_end: 7,
            peak_factor: 1.0,
            low_factor: 0.2,
        }
    }
}

enum Mapping {
    Card(usize),
    Gap(usize),
}

struct LedOutput {
    gpio: u8,
    start: usize,
    end: usize,
}

struct Button {
    gpio: u8,
    scene: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    size: f32,
}

struct CardDefinition {
    name: &'static str,
    image: &'static str,
    active: bool,
    positions: Vec<Position>,
    link: Vec<usize>,
    status: Option<usize>,
    labeled: BTreeMap<String, usize>,
}

impl CardDefinition {
    fn validate_catalog(&self, source_name: &str) -> Result<()> {
        let count = self.positions.len();
        for index in self
            .link
            .iter()
            .copied()
            .chain(self.status)
            .chain(self.labeled.values().copied())
        {
            if index >= count {
                return Err(BakeError::new(format!(
                    "internal linecard catalog error for {source_name:?}: LED index {index} exceeds count {count}"
                )));
            }
        }
        Ok(())
    }
}

fn position(x: i32, y: i32, size: i32) -> Position {
    Position {
        x: x as f32,
        y: y as f32,
        size: size as f32,
    }
}

fn labels(prefix: &str, indices: impl IntoIterator<Item = usize>) -> BTreeMap<String, usize> {
    indices
        .into_iter()
        .enumerate()
        .map(|(number, index)| (format!("{prefix}{}", number + 1), index))
        .collect()
}

fn card_definition(name: &str) -> Option<CardDefinition> {
    match name {
        "6478" => Some(card_6478()),
        "6704" => Some(card_6704()),
        "sup720" => Some(card_sup720()),
        "blank" => Some(card_blank()),
        "a9k-rsp400-se" => Some(card_rsp440()),
        "a9k-rsp400-se-2" => Some(card_rsp440_2()),
        "a9k-8t-l" => Some(card_a9k_8t()),
        "a9k-40ge-l" => Some(card_a9k_40ge()),
        _ => None,
    }
}

fn card_6478() -> CardDefinition {
    let ys = [
        73, 88, 101, 114, 128, 141, 154, 167, 180, 194, 207, 220, 233, 323, 336, 349, 362, 375,
        387, 400, 413, 426, 439, 451, 464, 549, 562, 575, 588, 601, 613, 626, 639, 652, 665, 677,
        690, 778, 791, 804, 817, 830, 842, 855, 868, 881, 894, 906, 919,
    ];
    let mut labeled = labels("p", 1..49);
    labeled.insert("status".to_owned(), 0);
    CardDefinition {
        name: "6478",
        image: "6478.png",
        active: true,
        positions: ys.into_iter().map(|y| position(11, y, 5)).collect(),
        link: (1..49).collect(),
        status: Some(0),
        labeled,
    }
}

fn card_6704() -> CardDefinition {
    let mut labeled = labels("p", 1..5);
    labeled.insert("status".to_owned(), 0);
    CardDefinition {
        name: "6704",
        image: "6704.png",
        active: true,
        positions: vec![
            position(27, 55, 8),
            position(12, 110, 5),
            position(12, 129, 5),
            position(12, 148, 5),
            position(12, 168, 5),
        ],
        link: (1..5).collect(),
        status: Some(0),
        labeled,
    }
}

fn card_sup720() -> CardDefinition {
    let mut labeled = labels("p", 6..9);
    for (name, index) in [
        ("status", 0),
        ("system", 1),
        ("active", 2),
        ("mgmt", 3),
        ("disk0", 4),
        ("disk1", 5),
    ] {
        labeled.insert(name.to_owned(), index);
    }
    CardDefinition {
        name: "sup720",
        image: "sup720.png",
        active: true,
        positions: vec![
            position(24, 57, 5),
            position(24, 71, 5),
            position(24, 85, 5),
            position(24, 98, 5),
            position(54, 105, 5),
            position(28, 291, 5),
            position(31, 579, 5),
            position(31, 652, 5),
            position(32, 725, 5),
        ],
        link: (6..9).collect(),
        status: Some(0),
        labeled,
    }
}

fn card_blank() -> CardDefinition {
    CardDefinition {
        name: "blank",
        image: "blank.png",
        active: false,
        positions: Vec::new(),
        link: Vec::new(),
        status: None,
        labeled: BTreeMap::new(),
    }
}

fn rsp_labels(entries: &[(&str, usize)]) -> BTreeMap<String, usize> {
    entries
        .iter()
        .map(|&(name, index)| (name.to_owned(), index))
        .collect()
}

fn rsp_positions() -> Vec<Position> {
    vec![
        position(57, 187, 5),
        position(57, 198, 5),
        position(57, 857, 4),
        position(57, 880, 4),
        position(57, 900, 4),
        position(43, 857, 4),
        position(43, 880, 4),
        position(43, 900, 4),
        position(30, 857, 4),
        position(30, 880, 4),
        position(30, 900, 4),
    ]
}

fn card_rsp440() -> CardDefinition {
    CardDefinition {
        name: "A9K-RSP440-SE",
        image: "a9k-rsp440-se.png",
        active: true,
        positions: rsp_positions(),
        link: vec![0],
        status: None,
        labeled: rsp_labels(&[
            ("fail", 8),
            ("crit", 5),
            ("sso", 2),
            ("aco", 9),
            ("maj", 6),
            ("fc_fault", 3),
            ("sync", 10),
            ("min", 7),
            ("gps", 4),
        ]),
    }
}

fn card_rsp440_2() -> CardDefinition {
    CardDefinition {
        name: "A9K-RSP440-SE-2",
        image: "a9k-rsp440-se.png",
        active: true,
        positions: vec![
            position(57, 187, 5),
            position(57, 198, 5),
            position(57, 857, 4),
            position(43, 880, 4),
            position(57, 880, 4),
            position(57, 900, 4),
            position(43, 857, 4),
            position(30, 880, 4),
            position(43, 900, 4),
            position(30, 857, 4),
            position(30, 900, 4),
        ],
        link: vec![0],
        status: None,
        labeled: rsp_labels(&[
            ("fail", 9),
            ("crit", 6),
            ("sso", 2),
            ("aco", 7),
            ("maj", 3),
            ("fc_fault", 4),
            ("sync", 10),
            ("min", 8),
            ("gps", 5),
        ]),
    }
}

fn card_a9k_8t() -> CardDefinition {
    CardDefinition {
        name: "A9K-8T-L",
        image: "a9k-8t-l.png",
        active: true,
        positions: vec![
            position(75, 73, 5),
            position(76, 180, 5),
            position(76, 280, 5),
            position(76, 380, 5),
            position(76, 530, 5),
            position(76, 635, 5),
            position(76, 740, 5),
            position(76, 845, 5),
            position(73, 985, 5),
        ],
        link: (0..8).collect(),
        status: Some(8),
        // Preserve the original Go catalog: p1..p8 point at LEDs 1..8.
        labeled: {
            let mut labeled = labels("p", 1..9);
            labeled.insert("status".to_owned(), 8);
            labeled
        },
    }
}

fn card_a9k_40ge() -> CardDefinition {
    let mut labeled = labels("p", 1..41);
    labeled.insert("status".to_owned(), 0);
    CardDefinition {
        name: "A9K-40GE-L",
        image: "a9k-40ge-l.png",
        active: true,
        positions: vec![
            position(75, 985, 5),
            position(56, 78, 5),
            position(56, 99, 5),
            position(57, 117, 5),
            position(57, 138, 5),
            position(57, 156, 5),
            position(57, 177, 5),
            position(57, 195, 5),
            position(57, 216, 5),
            position(57, 233, 5),
            position(57, 254, 5),
            position(57, 290, 5),
            position(57, 311, 5),
            position(57, 329, 5),
            position(57, 350, 5),
            position(57, 368, 5),
            position(57, 389, 5),
            position(57, 407, 5),
            position(57, 428, 5),
            position(57, 446, 5),
            position(57, 467, 5),
            position(56, 531, 5),
            position(56, 552, 5),
            position(56, 570, 5),
            position(56, 591, 5),
            position(56, 609, 5),
            position(56, 630, 5),
            position(56, 648, 5),
            position(56, 669, 5),
            position(56, 687, 5),
            position(56, 708, 5),
            position(56, 744, 5),
            position(56, 765, 5),
            position(56, 783, 5),
            position(56, 804, 5),
            position(56, 822, 5),
            position(56, 843, 5),
            position(56, 861, 5),
            position(56, 882, 5),
            position(56, 900, 5),
            position(56, 921, 5),
        ],
        link: (1..41).collect(),
        status: Some(0),
        labeled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("configurations")
            .join(name)
    }

    #[test]
    fn bakes_9010_catalog_and_outputs() {
        let baked = bake_path(config_path("9010.toml")).unwrap();
        assert_eq!(baked.led_count, 200);
        assert_eq!(baked.cards.len(), 10);
        assert_eq!(baked.cards[0].name, "A9K-40GE-L");
        assert_eq!(baked.cards[0].positions.len(), 41);
        assert_eq!(baked.cards[4].labeled["fail"], 8);
        assert_eq!(baked.cards[5].labeled["fail"], 9);
        assert_eq!(baked.cards[5].positions[3], position(43, 880, 4));
        assert_eq!(baked.cards[5].positions[7], position(30, 880, 4));
        assert_eq!(baked.led_type, "WS2815");
        assert_eq!(baked.led_outputs.len(), 2);
        assert_eq!(baked.led_outputs[1].end, 200);
        assert_eq!(baked.buttons[0].scene[3], "a9k-rsp440-tr");
        let rendered = baked.render();
        syn::parse_file(&rendered).expect("rendered 9010 configuration must be valid Rust");
        assert!(rendered.contains("pub const LED_COUNT: usize = 200;"));
    }

    #[test]
    fn bakes_7609_catalog_and_mapping() {
        let baked = bake_path(config_path("7609.toml")).unwrap();
        assert_eq!(baked.led_count, 132);
        assert_eq!(baked.cards.len(), 9);
        assert_eq!(baked.cards[0].positions.len(), 49);
        assert_eq!(baked.cards[4].positions.len(), 9);
        assert_eq!(baked.cards[8].labeled["p48"], 48);
        assert_eq!(baked.led_type, "APA102");
        assert_eq!(baked.led_outputs.len(), 1);
        assert_eq!(baked.mapping.len(), 11);
        let rendered = baked.render();
        syn::parse_file(&rendered).expect("rendered 7609 configuration must be valid Rust");
        assert!(rendered.contains("MappingSegment::Gap(1)"));
    }

    #[test]
    fn rejects_unknown_patterns_in_buttons() {
        let input = r#"
            LEDAmount = 1
            Linecards = ["blank"]
            Mapping = [{ gen = 1 }]
            Buttons = [{ pin = "GPIO1", action = ["not-a-pattern"] }]
        "#;
        let error = bake_text(Path::new("bad.toml"), input)
            .err()
            .expect("configuration should be rejected");
        assert!(error.to_string().contains("Buttons[0].action[0]"));
    }

    #[test]
    fn rejects_ambiguous_mapping_entries() {
        let input = r#"
            LEDAmount = 1
            Linecards = ["blank"]
            Mapping = [{ card = 0, gen = 1 }]
        "#;
        let error = bake_text(Path::new("bad.toml"), input)
            .err()
            .expect("configuration should be rejected");
        assert!(error.to_string().contains("Mapping[0]"));
    }

    #[test]
    fn rejects_unknown_cards_and_missing_required_fields() {
        let unknown = r#"
            LEDAmount = 1
            Linecards = ["mystery-card"]
        "#;
        let error = bake_text(Path::new("bad.toml"), unknown)
            .err()
            .expect("configuration should be rejected");
        assert!(error.to_string().contains("Linecards[0]"));

        let missing = r#"Linecards = ["blank"]"#;
        let error = bake_text(Path::new("bad.toml"), missing)
            .err()
            .expect("configuration should be rejected");
        assert!(error.to_string().contains("LEDAmount"));
    }

    #[test]
    fn rejects_bad_led_ranges_and_types() {
        let malformed = r#"
            LEDAmount = 10
            Linecards = ["blank"]
            [ledinfo]
            type = "WS2815"
            mapping = [{ gpio = 5, range = "nope" }]
        "#;
        let error = bake_text(Path::new("bad.toml"), malformed)
            .err()
            .expect("configuration should be rejected");
        assert!(error.to_string().contains("ledinfo.mapping[0].range"));

        let unsupported = r#"
            LEDAmount = 10
            Linecards = ["blank"]
            [ledinfo]
            type = "LASERBEAM"
        "#;
        let error = bake_text(Path::new("bad.toml"), unsupported)
            .err()
            .expect("configuration should be rejected");
        assert!(error.to_string().contains("ledinfo.type"));
    }
}
