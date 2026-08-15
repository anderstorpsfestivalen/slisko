use alloc::vec::Vec;
use core::fmt;

use crate::chassi::Chassi;
use crate::pixel::Pixel;

/// One segment of the configured logical-chassis to physical-strand mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingSegment {
    /// Append every LED belonging to the given chassis card index.
    Card(usize),
    /// Append this many black physical pixels without a logical LED.
    Gap(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingError {
    UnknownCard(usize),
    TooManyPixels { mapped: usize, capacity: usize },
    InvalidSrgbLength { expected: usize, actual: usize },
}

impl fmt::Display for MappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MappingError::UnknownCard(card) => write!(f, "mapping references unknown card {card}"),
            MappingError::TooManyPixels { mapped, capacity } => write!(
                f,
                "mapping expands to {mapped} pixels but the strand has {capacity}"
            ),
            MappingError::InvalidSrgbLength { expected, actual } => {
                write!(f, "sRGB frame has {actual} bytes, expected {expected}")
            }
        }
    }
}

/// Reusable mapping between logical chassis pixels and the physical strand.
///
/// The vector stores one optional logical LED index per physical output pixel.
/// Unmapped and generated positions are `None` and therefore render black.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrandMap {
    physical_to_logical: Vec<Option<usize>>,
}

impl StrandMap {
    pub fn new(
        chassi: &Chassi,
        segments: &[MappingSegment],
        led_count: usize,
    ) -> Result<Self, MappingError> {
        let mut physical_to_logical = Vec::with_capacity(led_count);
        for segment in segments {
            match *segment {
                MappingSegment::Card(card_idx) => {
                    let card = chassi
                        .linecards
                        .get(card_idx)
                        .ok_or(MappingError::UnknownCard(card_idx))?;
                    physical_to_logical
                        .extend((card.led_offset..card.led_offset + card.led_count).map(Some));
                }
                MappingSegment::Gap(count) => {
                    physical_to_logical.extend(core::iter::repeat_n(None, count));
                }
            }
            if physical_to_logical.len() > led_count {
                return Err(MappingError::TooManyPixels {
                    mapped: physical_to_logical.len(),
                    capacity: led_count,
                });
            }
        }
        physical_to_logical.resize(led_count, None);
        Ok(Self {
            physical_to_logical,
        })
    }

    pub fn len(&self) -> usize {
        self.physical_to_logical.len()
    }

    pub fn is_empty(&self) -> bool {
        self.physical_to_logical.is_empty()
    }

    /// Return every physical pixel exactly once, grouped by a requested
    /// logical card order. Unmapped pixels immediately preceding a card in the
    /// physical map follow that card's logical pixels; trailing padding follows
    /// all card groups. This keeps real card LEDs first while still covering
    /// every installed pixel.
    pub fn physical_order_by_cards(&self, chassi: &Chassi, card_order: &[usize]) -> Vec<usize> {
        let mut groups = alloc::vec![Vec::new(); chassi.linecards.len()];
        let mut leading_unmapped = alloc::vec![Vec::new(); chassi.linecards.len()];
        let mut pending_unmapped = Vec::new();
        let mut trailing = Vec::new();

        for (physical, logical) in self.physical_to_logical.iter().copied().enumerate() {
            let Some(logical) = logical else {
                pending_unmapped.push(physical);
                continue;
            };
            let card = chassi.linecards.iter().position(|candidate| {
                (candidate.led_offset..candidate.led_offset + candidate.led_count)
                    .contains(&logical)
            });
            let Some(card) = card else {
                trailing.append(&mut pending_unmapped);
                trailing.push(physical);
                continue;
            };
            leading_unmapped[card].append(&mut pending_unmapped);
            groups[card].push(physical);
        }
        trailing.append(&mut pending_unmapped);

        let mut included = alloc::vec![false; groups.len()];
        let mut ordered = Vec::with_capacity(self.len());
        for &card in card_order {
            if card < groups.len() && !included[card] {
                ordered.append(&mut groups[card]);
                ordered.append(&mut leading_unmapped[card]);
                included[card] = true;
            }
        }
        for (card, group) in groups.iter_mut().enumerate() {
            if !included[card] {
                ordered.append(group);
                ordered.append(&mut leading_unmapped[card]);
            }
        }
        ordered.append(&mut trailing);
        ordered
    }

    /// Resolve selected logical LEDs to their physical pixel indices, keeping
    /// physical strand order and excluding generated/unmapped pixels.
    pub fn physical_indices_for_logical(&self, logical_indices: &[usize]) -> Vec<usize> {
        self.physical_to_logical
            .iter()
            .enumerate()
            .filter_map(|(physical, logical)| {
                logical
                    .is_some_and(|logical| logical_indices.contains(&logical))
                    .then_some(physical)
            })
            .collect()
    }

    /// Copy logical pixels into a reusable physical-pixel buffer without any
    /// device-specific transfer conversion.
    pub fn copy_pixels(&self, logical: &[Pixel], physical: &mut Vec<Pixel>) {
        physical.clear();
        physical.reserve(self.len());
        for logical_idx in &self.physical_to_logical {
            physical.push(
                logical_idx
                    .and_then(|idx| logical.get(idx).copied())
                    .unwrap_or_else(Pixel::new),
            );
        }
    }

    /// Encode the mapped strand as tightly packed logical sRGB8 bytes.
    pub fn encode_srgb8(&self, logical: &[Pixel], srgb: &mut Vec<u8>) {
        srgb.clear();
        srgb.reserve(self.len() * 3);
        for logical_idx in &self.physical_to_logical {
            let [r, g, b] = logical_idx
                .and_then(|idx| logical.get(idx))
                .map(Pixel::to_srgb8)
                .unwrap_or([0, 0, 0]);
            srgb.extend_from_slice(&[r, g, b]);
        }
    }

    /// Apply a complete logical sRGB8 strand frame back to the chassis. LEDs
    /// absent from the physical mapping are blanked.
    pub fn apply_srgb8(&self, srgb: &[u8], logical: &mut [Pixel]) -> Result<(), MappingError> {
        let expected = self.len() * 3;
        if srgb.len() != expected {
            return Err(MappingError::InvalidSrgbLength {
                expected,
                actual: srgb.len(),
            });
        }
        for pixel in logical.iter_mut() {
            pixel.set_color(0.0, 0.0, 0.0);
        }
        for (physical_idx, logical_idx) in self.physical_to_logical.iter().enumerate() {
            if let Some(pixel) = logical_idx.and_then(|idx| logical.get_mut(idx)) {
                let offset = physical_idx * 3;
                pixel.set_color(
                    srgb[offset] as f32 / 255.0,
                    srgb[offset + 1] as f32 / 255.0,
                    srgb[offset + 2] as f32 / 255.0,
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chassi::LineCardSpec;
    use crate::pixel::Position;

    fn px(r: f32, g: f32, b: f32) -> Pixel {
        let mut p = Pixel::new();
        p.set_color(r, g, b);
        p
    }

    static CARD_A: &[Position] = &[
        Position {
            x: 0.0,
            y: 0.0,
            size: 1.0,
        },
        Position {
            x: 1.0,
            y: 0.0,
            size: 1.0,
        },
    ];
    static CARD_B: &[Position] = &[Position {
        x: 2.0,
        y: 0.0,
        size: 1.0,
    }];
    static MAP_SPECS: &[LineCardSpec] = &[
        LineCardSpec {
            name: "A",
            image: "",
            active: true,
            positions: CARD_A,
            link: &[],
            status: None,
            labeled: &[],
        },
        LineCardSpec {
            name: "B",
            image: "",
            active: true,
            positions: CARD_B,
            link: &[],
            status: None,
            labeled: &[],
        },
    ];

    #[test]
    fn reorders_gaps_and_pads_logical_srgb() {
        let mut chassi = Chassi::from_specs(MAP_SPECS);
        chassi.leds[0] = px(1.0, 0.0, 0.0);
        chassi.leds[1] = px(0.0, 1.0, 0.0);
        chassi.leds[2] = px(0.0, 0.0, 1.0);
        let map = StrandMap::new(
            &chassi,
            &[
                MappingSegment::Card(1),
                MappingSegment::Gap(1),
                MappingSegment::Card(0),
            ],
            5,
        )
        .unwrap();
        let mut srgb = Vec::new();
        map.encode_srgb8(&chassi.leds, &mut srgb);
        assert_eq!(
            srgb,
            alloc::vec![0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 0]
        );
    }

    #[test]
    fn physical_order_groups_leading_gaps_with_the_following_card() {
        let chassi = Chassi::from_specs(MAP_SPECS);
        let map = StrandMap::new(
            &chassi,
            &[
                MappingSegment::Card(1),
                MappingSegment::Gap(1),
                MappingSegment::Card(0),
            ],
            5,
        )
        .unwrap();

        assert_eq!(
            map.physical_order_by_cards(&chassi, &[0, 1]),
            [2, 3, 1, 0, 4]
        );
        assert_eq!(map.physical_indices_for_logical(&[0, 2]), [0, 2]);
    }

    #[test]
    fn round_trips_srgb_without_physical_transfer_conversion() {
        let chassi = Chassi::from_specs(MAP_SPECS);
        let map = StrandMap::new(&chassi, &[MappingSegment::Card(1)], 1).unwrap();
        let mut target = chassi.leds.clone();
        for pixel in &mut target {
            pixel.set_color(1.0, 1.0, 1.0);
        }
        map.apply_srgb8(&[10, 20, 30], &mut target).unwrap();
        assert_eq!(target[0].to_srgb8(), [0, 0, 0]);
        assert_eq!(target[1].to_srgb8(), [0, 0, 0]);
        assert_eq!(target[2].to_srgb8(), [10, 20, 30]);
    }

    #[test]
    fn rejects_bad_config_and_frame_lengths() {
        let chassi = Chassi::from_specs(MAP_SPECS);
        assert_eq!(
            StrandMap::new(&chassi, &[MappingSegment::Card(9)], 2),
            Err(MappingError::UnknownCard(9))
        );
        assert_eq!(
            StrandMap::new(&chassi, &[MappingSegment::Card(0)], 1),
            Err(MappingError::TooManyPixels {
                mapped: 2,
                capacity: 1
            })
        );
        let map = StrandMap::new(&chassi, &[MappingSegment::Gap(1)], 1).unwrap();
        let mut pixels = chassi.leds.clone();
        assert_eq!(
            map.apply_srgb8(&[], &mut pixels),
            Err(MappingError::InvalidSrgbLength {
                expected: 3,
                actual: 0
            })
        );
    }
}
