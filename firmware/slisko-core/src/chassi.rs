//! Ported from `pkg/chassi` (`chassi.go`, `linecard.go`).
//!
//! ## Port note: indices instead of pointers
//!
//! The Go model owns pixels in each `LineCard.LEDs` slice and aliases them with
//! raw pointers (`Link []*Pixel`, `Status *Pixel`, `Labeled map[string]*Pixel`,
//! and `Chassi.LEDs []*Pixel`). That shared-mutable aliasing doesn't translate
//! to Rust. Instead, [`Chassi`] owns **one flat `Vec<Pixel>`** (the strand) and
//! every other reference is a `usize` index into it. Patterns mutate
//! `chassi.leds[i]`; helpers hand back the index lists a pattern needs.
//!
//! The per-card layout (LED count, positions, link/status/label assignments) is
//! pure data emitted by the Go `cmd/baker` as [`LineCardSpec`] literals; all the
//! logic lives here.

use alloc::vec::Vec;

use crate::pixel::{Pixel, Position};

/// Static description of one linecard, emitted by the baker. Indices in
/// `link`, `status`, and `labeled` are **card-local** (0-based within this
/// card's LEDs); [`Chassi::from_specs`] converts them to absolute strand
/// indices.
#[derive(Clone, Copy, Debug)]
pub struct LineCardSpec {
    pub name: &'static str,
    pub image: &'static str,
    pub active: bool,
    /// One position per LED on the card; its length is the card's LED count.
    pub positions: &'static [Position],
    /// Card-local indices that are "link" port LEDs.
    pub link: &'static [usize],
    /// Card-local index of the status LED, if any.
    pub status: Option<usize>,
    /// `(label, card-local index)` pairs (e.g. `("fail", 8)`).
    pub labeled: &'static [(&'static str, usize)],
}

/// Runtime linecard. All indices are absolute into [`Chassi::leds`].
#[derive(Clone, Debug)]
pub struct LineCard {
    pub name: &'static str,
    pub image: &'static str,
    pub active: bool,
    pub led_offset: usize,
    pub led_count: usize,
    pub status: Option<usize>,
    pub link: Vec<usize>,
    pub labeled: Vec<(&'static str, usize)>,
}

/// The full chassis: owns the flat strand and the linecard metadata.
#[derive(Clone, Debug, Default)]
pub struct Chassi {
    pub leds: Vec<Pixel>,
    pub linecards: Vec<LineCard>,
    link_ports: Vec<usize>,
    status_leds: Vec<usize>,
}

impl Chassi {
    /// Build a chassis from baked specs (mirrors `chassi.New` +
    /// `CardsFromDefinition` + position setup).
    pub fn from_specs(specs: &[LineCardSpec]) -> Self {
        let mut leds: Vec<Pixel> = Vec::new();
        let mut linecards: Vec<LineCard> = Vec::new();

        for s in specs {
            let offset = leds.len();
            let count = s.positions.len();
            for pos in s.positions {
                let mut p = Pixel::new();
                p.set_position(pos.x, pos.y, pos.size);
                leds.push(p);
            }

            let link: Vec<usize> = s.link.iter().map(|&i| offset + i).collect();
            let status = s.status.map(|i| offset + i);
            let labeled: Vec<(&'static str, usize)> =
                s.labeled.iter().map(|&(l, i)| (l, offset + i)).collect();

            linecards.push(LineCard {
                name: s.name,
                image: s.image,
                active: s.active,
                led_offset: offset,
                led_count: count,
                status,
                link,
                labeled,
            });
        }

        let mut c = Chassi {
            leds,
            linecards,
            link_ports: Vec::new(),
            status_leds: Vec::new(),
        };
        c.recompute_aggregates();
        c
    }

    /// Recompute the chassis-level link/status index lists from active cards
    /// (mirrors `getlinkPorts` / `getstatusLEDs`).
    fn recompute_aggregates(&mut self) {
        let mut link_ports = Vec::new();
        let mut status_leds = Vec::new();
        for lc in &self.linecards {
            if !lc.active {
                continue;
            }
            link_ports.extend_from_slice(&lc.link);
            if let Some(s) = lc.status {
                status_leds.push(s);
            }
        }
        self.link_ports = link_ports;
        self.status_leds = status_leds;
    }

    /// All link-port indices across active cards (mirrors `Chassi.LinkPorts`).
    pub fn link_ports(&self) -> &[usize] {
        &self.link_ports
    }

    /// All status-LED indices across active cards (mirrors `Chassi.StatusLEDs`).
    pub fn status_leds(&self) -> &[usize] {
        &self.status_leds
    }

    /// Linecard indices whose name matches (mirrors `GetCardOfType`).
    pub fn cards_of_type(&self, name: &str) -> Vec<usize> {
        self.linecards
            .iter()
            .enumerate()
            .filter(|(_, lc)| lc.name == name)
            .map(|(i, _)| i)
            .collect()
    }

    /// All link-port indices belonging to cards of the given type. This is the
    /// common pattern-bootstrap shape: `for card in GetCardOfType(t) { card.Link }`.
    pub fn link_indices_of_type(&self, name: &str) -> Vec<usize> {
        let mut out = Vec::new();
        for lc in self.linecards.iter().filter(|lc| lc.name == name) {
            out.extend_from_slice(&lc.link);
        }
        out
    }

    /// Strand indices carrying the given label across all cards (mirrors
    /// `GetLEDsWithLabel`).
    pub fn leds_with_label(&self, label: &str) -> Vec<usize> {
        let mut out = Vec::new();
        for lc in &self.linecards {
            for &(l, idx) in &lc.labeled {
                if l == label {
                    out.push(idx);
                }
            }
        }
        out
    }

    /// Card names in order (mirrors `GetCardOrder`).
    pub fn card_order(&self) -> Vec<&'static str> {
        self.linecards.iter().map(|lc| lc.name).collect()
    }

    /// The strand index of a label on a specific linecard, if present.
    pub fn card_label(&self, card_idx: usize, label: &str) -> Option<usize> {
        self.linecards
            .get(card_idx)?
            .labeled
            .iter()
            .find(|(l, _)| *l == label)
            .map(|&(_, i)| i)
    }

    /// Strand indices of `label` across all cards whose name matches `card_type`
    /// (scoped variant of [`leds_with_label`](Self::leds_with_label)).
    pub fn leds_with_label_on_type(&self, card_type: &str, label: &str) -> Vec<usize> {
        let mut out = Vec::new();
        for lc in self.linecards.iter().filter(|lc| lc.name == card_type) {
            for &(l, idx) in &lc.labeled {
                if l == label {
                    out.push(idx);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Two-card fixture modeled on the Go RSP440 layout (link at 0, a few labels).
    static CARD_A_POS: &[Position] = &[
        Position {
            x: 0.0,
            y: 0.0,
            size: 5.0,
        },
        Position {
            x: 1.0,
            y: 0.0,
            size: 5.0,
        },
        Position {
            x: 2.0,
            y: 0.0,
            size: 4.0,
        },
    ];
    static CARD_B_POS: &[Position] = &[
        Position {
            x: 0.0,
            y: 10.0,
            size: 5.0,
        },
        Position {
            x: 1.0,
            y: 10.0,
            size: 5.0,
        },
    ];

    static SPECS: &[LineCardSpec] = &[
        LineCardSpec {
            name: "CARD-A",
            image: "a.png",
            active: true,
            positions: CARD_A_POS,
            link: &[0, 1],
            status: Some(2),
            labeled: &[("fail", 2)],
        },
        LineCardSpec {
            name: "CARD-B",
            image: "b.png",
            active: false, // inactive -> excluded from aggregates
            positions: CARD_B_POS,
            link: &[0],
            status: None,
            labeled: &[("link0", 0)],
        },
    ];

    #[test]
    fn builds_flat_strand_and_positions() {
        let c = Chassi::from_specs(SPECS);
        assert_eq!(c.leds.len(), 5); // 3 + 2
        assert_eq!(c.linecards[1].led_offset, 3);
        // Card B's first pixel position is baked through.
        assert_eq!(c.leds[3].pos.y, 10.0);
    }

    #[test]
    fn link_and_status_indices_are_absolute() {
        let c = Chassi::from_specs(SPECS);
        // Card A link {0,1} stays {0,1}; card B link {0} -> absolute {3}.
        assert_eq!(c.link_indices_of_type("CARD-A"), alloc::vec![0, 1]);
        assert_eq!(c.link_indices_of_type("CARD-B"), alloc::vec![3]);
        // Aggregates only include the active card (A).
        assert_eq!(c.link_ports(), &[0, 1]);
        assert_eq!(c.status_leds(), &[2]);
    }

    #[test]
    fn labels_resolve_to_absolute_indices() {
        let c = Chassi::from_specs(SPECS);
        assert_eq!(c.leds_with_label("fail"), alloc::vec![2]);
        assert_eq!(c.leds_with_label("link0"), alloc::vec![3]); // offset 3 + local 0
        assert!(c.leds_with_label("nope").is_empty());
    }

    #[test]
    fn card_order_and_types() {
        let c = Chassi::from_specs(SPECS);
        assert_eq!(c.card_order(), alloc::vec!["CARD-A", "CARD-B"]);
        assert_eq!(c.cards_of_type("CARD-B"), alloc::vec![1]);
    }
}
