use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Default for Tone {
    fn default() -> Self {
        Self::C
    }
}

impl PartialOrd for Tone {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Tone {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quarter_delta().cmp(&other.quarter_delta())
    }
}

impl Display for Tone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
        f,
            "{}",
            match self {
                Self::A => 'A',
                Self::B => 'B',
                Self::C => 'C',
                Self::D => 'D',
                Self::E => 'E',
                Self::F => 'F',
                Self::G => 'G',
            }
        )
    }
}

impl FromStr for Tone {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();
        let mut iter = trimmed.chars();
        let raw = iter.next();
        if raw == None || iter.next() != None {
            Err(())
        } else {
            match raw.unwrap() {
                'a' => Ok(Self::A),
                'b' => Ok(Self::B),
                'c' => Ok(Self::C),
                'd' => Ok(Self::D),
                'e' => Ok(Self::E),
                'f' => Ok(Self::F),
                'g' => Ok(Self::G),
                _ => Err(()),
            }
        }
    }
}

impl Tone {
    /// The number of semitones above C
    pub fn semitone_delta(&self) -> u32 {
        match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11
        }
    }

    /// The number of quarter tones above C
    pub fn quarter_delta(&self) -> u32 {
        self.semitone_delta() * 2
    }

    /// The number of cents above c
    pub fn cent_delta(&self) -> u32 {
        self.semitone_delta() * 100
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
    QtrFlat,
    Flat,
    ThreeQtrFlat,
    Natural,
    QtrSharp,
    Sharp,
    ThreeQtrSharp,
}

impl Default for Accidental {
    fn default() -> Self {
        Self::Natural
    }
}

impl PartialOrd for Accidental {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Accidental {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quarter_delta().cmp(&other.quarter_delta())
    }
}

impl Display for Accidental {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
        f,
            "{}",
            match self {
                Self::ThreeQtrFlat => "_b",
                Self::Flat => "b",
                Self::QtrFlat => "^b",
                Self::Natural => "",
                Self::QtrSharp => "_#",
                Self::Sharp => "#",
                Self::ThreeQtrSharp => "^#",
            }
        )
    }
}

impl Accidental {
    /// The number of quarter tones a pitch would be changed by
    pub fn quarter_delta(&self) -> i32 {
        match self {
            Self::QtrFlat => -3,
            Self::Flat => -2,
            Self::ThreeQtrFlat => -1,
            Self::Natural => 0,
            Self::QtrSharp => 1,
            Self::Sharp => 2,
            Self::ThreeQtrSharp => 3,
        }
    }

    /// The number of cents a pitch would be changed by
    pub fn cent_delta(&self) -> i32 {
        self.quarter_delta() * 50
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct Pitch {
    pub octave: u8,
    pub tone: Tone,
    pub accidental: Accidental,
}

impl Default for Pitch {
    fn default() -> Self {
        Self {
            tone: Tone::C,
            accidental: Accidental::Natural,
            octave: 4
        }
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Pitch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quarter_delta_c0_34b().cmp(&other.quarter_delta_c0_34b())
    }
}

impl PartialEq for Pitch {
    fn eq(&self, other: &Self) -> bool {
        self.quarter_delta_c0_34b() == other.quarter_delta_c0_34b()
    }
}

impl Eq for Pitch {}

#[derive(Debug, Error)]
pub enum PitchParseError {
    #[error("Missing tone. Tone must be the first character listed.")]
    MissingTone,

    #[error("Unrecognized tone '{0}'. Must be a letter A-G (non-case sensitive).")]
    UnrecognizedTone(String),

    #[error("Unrecognized accidental '{0}'. Must be either unspecified or one of 'b' or '#', possibly preceeded by '_' or '^'.")]
    UnrecognizedAccidental(String),

    #[error("Missing octave. Octave must be specified at the very end of the string.")]
    MissingOctave,

    #[error("Unable to parse octave '{0}'. Octave must be an integer between 0 and 255, inclusive")]
    UnrecognizedOctave(String),

    #[error("Octave must be an integer between 0 and 255, inclusive")]
    OctaveOverflow
}

impl FromStr for Pitch {
    type Err = PitchParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let mut iter = trimmed.chars().peekable();

        // parse tone
        let tone_unchecked = {
            let raw = iter.next();
            if raw == None {
                return Err(PitchParseError::MissingTone);
            }
            raw.unwrap()
        };
        let tone = match tone_unchecked.to_lowercase().next() {
            Some('a') => Tone::A,
            Some('b') => Tone::B,
            Some('c') => Tone::C,
            Some('d') => Tone::D,
            Some('e') => Tone::E,
            Some('f') => Tone::F,
            Some('g') => Tone::G,
            Some(t) => { return Err(PitchParseError::UnrecognizedTone(t.to_string())) }
        	None => { return Err(PitchParseError::MissingTone) }
        };

        let octave_or_accidental = {
            let raw = iter.next();
            if raw == None {
                return Err(PitchParseError::MissingOctave);
            }
            raw.unwrap()
        };

        let mut accidental = Accidental::Natural;
        let mut octave: u32 = 0;

        // parse non-natural accidental
        if "_^b#".contains(octave_or_accidental) {
            if octave_or_accidental == 'b' {
                accidental = Accidental::Flat;
            } else if octave_or_accidental == '#' {
                accidental = Accidental::Sharp;
            } else {
                let modifier = octave_or_accidental;
                let base = {
                    let raw = iter.next();
                    match raw {
                        Some('b') | Some('#') => raw.unwrap(),
                        Some(t) => {
                            return Err(PitchParseError::UnrecognizedAccidental(
                                (octave_or_accidental.to_string() + &t.to_string()).to_string())
                            );
                        }
                        None => {
                            return Err(PitchParseError::UnrecognizedAccidental(
                                octave_or_accidental.to_string())
                            );
                        }
                    }
                };

                accidental = match (modifier, base) {
                    ('_', 'b') => Accidental::ThreeQtrFlat,
                    ('^', 'b') => Accidental::QtrFlat,
                    ('_', '#') => Accidental::QtrSharp,
                    ('^', '#') => Accidental::ThreeQtrSharp,
                    _ => unreachable!("parse unreachable")
                }
            }
            if iter.peek() == None {
                return Err(PitchParseError::MissingOctave);
            }
        } else if octave_or_accidental.is_digit(10) {
            octave = octave_or_accidental.to_digit(10).unwrap();
        } else {
            return Err(PitchParseError::MissingOctave);
        }

        while let Some(character) = iter.next() {
            if let Some(digit) = character.to_digit(10) {
                octave *= 10;
                octave += digit;
                if octave > (u8::MAX as u32) {
                    return Err(PitchParseError::OctaveOverflow)
                }
            } else {
                break;
            }
        }

        Ok(Self {
            tone: tone,
            accidental: accidental,
            octave: octave as u8
        })
    }
}

impl Display for Pitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.tone,
            self.accidental,
            self.octave
        )
    }
}

impl Pitch {
    /// The number of whole tones per octave
    pub const TONES_PER_OCTAVE: u32 = 7;

    /// The number of distinct half tones per octave
    pub const SEMITONES_PER_OCTAVE: u32 = 12;

    /// The number of quarter tones per octave
    pub const MICROTONES_PER_OCTAVE: u32 = Self::SEMITONES_PER_OCTAVE * 2;

    /// The number of cents per quarter tone
    pub const CENTS_PER_MICROTONE: u32 = 50;

    /// The number of cents per semitone
    pub const CENTS_PER_SEMITONE: u32 = Self::CENTS_PER_MICROTONE * 2;

    /// The number of cents per whole tone
    pub const CENTS_PER_TONE: u32 = Self::CENTS_PER_SEMITONE * 2;

    /// The number of cents in an octave
    pub const CENTS_PER_OCTAVE: u32 = Self::SEMITONES_PER_OCTAVE * Self::CENTS_PER_SEMITONE;

    /// The index of the note relative to C0 ThreeQtrFlat. Each quartertone has
    /// a unique index.
    pub fn quarter_delta_c0_34b(&self) -> u32 {
        ((3 // adjust for C0 ThreeQtrFlat being the lowest representable note
        + (self.octave as u32) * Self::MICROTONES_PER_OCTAVE // incorporate octaves below this one
        + self.tone.quarter_delta()) as i32 // handle tone
        + self.accidental.quarter_delta()) as u32 // handle accidental
    }

    /// Gets the number of quarters a given pitch is from A4
    pub fn quarter_delta_a4(&self) -> i32 {
        let a_quarters = Pitch {
            tone: Tone::A,
            accidental: Accidental::Natural,
            octave: 4
        }.quarter_delta_c0_34b() as i32;
        let pitch_quarters = self.quarter_delta_c0_34b() as i32;
        pitch_quarters - a_quarters
    }

    /// gets the number of cents a given pitch is for c0 ThreeQtrFlat
    pub fn cent_delta_c0_34b(&self) -> u32 {
        self.quarter_delta_c0_34b() * Self::CENTS_PER_MICROTONE
    }

    /// Gets the number of cents a given pitch is from A4
    pub fn cent_delta_a4(&self) -> i32 {
        self.quarter_delta_a4() * Self::CENTS_PER_MICROTONE as i32
    }

    /// Get the frequency of the pitch using the given tuning system
    pub fn frequency(&self, tuning_system: TuningSystem, detune: i32) -> f64 {
        tuning_system.get_pitch_frequency(&self, detune)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DetunedPitch {
    /// base pitch
    pub base_pitch: Pitch,

    /// detune of the pitch in cents
    /// i8 is chosen as there is only 100 cents between pitches
    /// so for detunes 100 cents or greater, we can represent that
    /// by changing the base pitch
    pub detune: i8
}

impl Display for DetunedPitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, +{:.2}c", self.base_pitch, self.detune)
    }
}

impl DetunedPitch {
    pub fn cent_delta_c0_34b(&self) -> i32 {
        self.base_pitch.cent_delta_c0_34b() as i32 + self.detune as i32
    }

    pub fn cent_delta_a4(&self) -> i32 {
        self.base_pitch.cent_delta_a4() + self.detune as i32
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TuningSystem {
    /// Twelve-tone equal temperment. Value contains pitch of A4.
    EqualTemperment(f64)
}

impl TuningSystem {
    pub fn get_pitch_frequency(&self, pitch: &Pitch, detune: i32) -> f64 {
        match self {
            Self::EqualTemperment(a4) => equal_temperment::get_pitch_frequency(*a4, pitch, detune),
        }
    }
}

pub mod equal_temperment {
    use super::*;

    /// gets the frequency given the difference in cents from a4
    pub fn get_cent_delta_a4_frequency(a4: f64, cents: f64) -> f64 {
        a4 as f64 * 2.0_f64.powf(cents / Pitch::CENTS_PER_OCTAVE as f64)
    }

    /// Gets the frequency of the given pitch, given the frequency of A4
    pub fn get_pitch_frequency(a4: f64, pitch: &Pitch, detune: i32) -> f64 {
        get_cent_delta_a4_frequency(a4, pitch.cent_delta_a4() as f64 + detune as f64)
    }

    /// quantizes x to the nearest half tone frequency
    /// Assumes x is greater than zero
    pub fn quantize_semitone(a4: f64, x: f64) -> f64 {
        let quantize_index = f64::round(12.0 * f64::log2(x / a4));
        a4 * f64::powf(2.0, quantize_index / 12.0)
    }

    /// quantizes x to the nearest quarter tone frequency
    /// Assumes x is greater than zero
    pub fn quantize_microtone(a4: f64, x: f64) -> f64 {
        let quantize_index = f64::round(24.0 * f64::log2(x / a4));
        a4 * f64::powf(2.0, quantize_index / 24.0)
    }

    /// quantizes x to the nearest major scale note of the given root
    pub fn quantize_major_scale(root: f64, x: f64) -> f64 {
        // picker function
        fn p(x: f64) -> f64 {
            f64::min(1.0, (x + 12.0) % 12.0)
        }
        let i = f64::round(12.0 * f64::log2(x / root));
        let s = i + p(i - 1.0) + p(i - 3.0) + p(i - 6.0) + p(i - 8.0) + p(i - 10.0) - 5.0;
        root * f64::powf(2.0, s / 12.0)
    }
}
