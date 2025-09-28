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
    /// The number of quarter tones above C
    pub fn quarter_delta(&self) -> u32 {
        match self {
            Self::C => 0,
            Self::D => 4,
            Self::E => 8,
            Self::F => 10,
            Self::G => 14,
            Self::A => 18,
            Self::B => 22,
        }
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
        self.quarters().cmp(&other.quarters())
    }
}

impl PartialEq for Pitch {
    fn eq(&self, other: &Self) -> bool {
        self.quarters() == other.quarters()
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
    pub const MICROTONES_PER_OCTAVE: u32 = 24;

    /// The number of cents in an octave
    pub const CENTS_PER_OCTAVE: u32 = 1200;

    /// The number of cents per whole tone
    pub const CENTS_PER_TONE: u32 = 200;

    /// The number of cents per semitone
    pub const CENTS_PER_SEMITONE: u32 = 100;

    /// The number of cents per quarter tone
    pub const CENTS_PER_MICROTONE: u32 = 50;

    /// The index of the note relative to C0 ThreeQtrFlat. Each quartertone has
    /// a unique index.
    pub fn quarters(&self) -> u32 {
        ((3 // adjust for C0 ThreeQtrFlat being the lowest representable note
        + (self.octave as u32) * Self::MICROTONES_PER_OCTAVE // incorporate octaves below this one
        + self.tone.quarter_delta()) as i32 // handle tone
        + self.accidental.quarter_delta()) as u32 // handle accidental
    }

    /// Gets the number of quarters a given pitch is from A4
    pub fn quarter_delta(&self) -> i32 {
        let a_quarters = Pitch {
            tone: Tone::A,
            accidental: Accidental::Natural,
            octave: 4
        }.quarters() as i32;
        let pitch_quarters = self.quarters() as i32;
        pitch_quarters - a_quarters
    }

    /// Gets the number of cents a given pitch is from A4
    pub fn cent_delta(&self) -> i32 {
        self.quarter_delta() * Self::CENTS_PER_MICROTONE as i32
    }

    /// Get the frequency of the pitch using the given tuning system
    pub fn frequency<T: TuningSystem>(&self, tuning_system: T) -> f32 {
        tuning_system.get_pitch_frequency(&self)
    }
}

pub trait TuningSystem {
    fn get_pitch_frequency(&self, pitch: &Pitch) -> f32;
}

/// Twelve tone equal temperment tuning system
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct EqualTemperment {
    /// The pitch of A4
    pub a4_pitch: f32
}

impl TuningSystem for EqualTemperment {
    fn get_pitch_frequency(&self, pitch: &Pitch) -> f32 {
        (self.a4_pitch as f64 * 2.0_f64.powf(
            pitch.quarter_delta() as f64
            / Pitch::MICROTONES_PER_OCTAVE as f64
        )) as f32
    }
}

impl EqualTemperment {
    pub fn new(a4_pitch: f32) -> Self {
        Self {
            a4_pitch
        }
    }
}

impl Default for EqualTemperment {
    fn default() -> Self {
        Self::new(440.0)
    }
}

