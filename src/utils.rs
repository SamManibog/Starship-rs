use std::{fmt::Display, str::FromStr};

use egui::Ui;

use crate::pitch::Pitch;

#[derive(Debug, Clone, Copy)]
pub enum PitchOrValue<T> {
    Value(T),
    Pitch(Pitch)
}

impl<T: Display> Display for PitchOrValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Value(val) => val.to_string(),
                Self::Pitch(pitch) => pitch.to_string(),
            }
        )
    }
}

pub fn pitch_or_number_input<T>(
    ui: &mut Ui,
    text: &mut String,
    value: &mut PitchOrValue<T>
) where T: FromStr + Display {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        *text = new_text;
    }

    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<Pitch>() {
            *value = PitchOrValue::Pitch(new_value);
        } else if let Ok(new_value) = text.parse::<T>() {
            *value = PitchOrValue::Value(new_value)
        }
        *text = value.to_string();
    }
}

pub fn pitch_or_pos_number_input<T>(
    ui: &mut Ui,
    text: &mut String,
    value: &mut PitchOrValue<T>
) where T: FromStr + Display + PositiveCheckable {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        *text = new_text;
    }

    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<Pitch>() {
            *value = PitchOrValue::Pitch(new_value);
        } else if let Ok(new_value) = text.parse::<T>() {
            if new_value.is_positive() {
                *value = PitchOrValue::Value(new_value)
            }
        }
        *text = value.to_string();
    }
}

pub fn pitch_or_non_neg_number_input<T>(
    ui: &mut Ui,
    text: &mut String,
    value: &mut PitchOrValue<T>
) where T: FromStr + Display + NonNegativeCheckable {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        *text = new_text;
    }

    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<Pitch>() {
            *value = PitchOrValue::Pitch(new_value);
        } else if let Ok(new_value) = text.parse::<T>() {
            if new_value.is_non_negative() {
                *value = PitchOrValue::Value(new_value)
            }
        }
        *text = value.to_string();
    }
}


pub fn number_input<T>(
    ui: &mut Ui,
    text: &mut String,
    value: &mut T
) where T: FromStr + Display {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        *text = new_text;
    }

    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<T>() {
            *value = new_value;
        }
        *text = value.to_string();
    }
}

pub fn pos_number_input<T>(
    ui: &mut Ui,
    text: &mut String,
    value: &mut T
) where T: FromStr + Display + PositiveCheckable {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        *text = new_text;
    }

    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<T>() {
            if new_value.is_positive() {
                *value = new_value;
            }
        }
        *text = value.to_string();
    }
}

pub fn non_neg_number_input<T>(
    ui: &mut Ui,
    text: &mut String,
    value: &mut T
) where T: FromStr + Display + NonNegativeCheckable {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        *text = new_text;
    }

    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<T>() {
            if new_value.is_non_negative() {
                *value = new_value;
            }
        }
        *text = value.to_string();
    }
}

/// Trait used for input utils. Ensures you can check if a value is positive.
pub trait PositiveCheckable {
    fn is_positive(&self) -> bool;
}

macro_rules! positive_checkable_impl {
    ($($t:ty),*) => {
    $(
    	impl PositiveCheckable for $t {
    		fn is_positive(&self) -> bool {
    			*self > (0 as Self)
    		}
    	}
    )*
    };
}

positive_checkable_impl!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

/// Trait used for input utils. Ensures you can check if a value is non-negative.
pub trait NonNegativeCheckable {
    fn is_non_negative(&self) -> bool;
}

macro_rules! non_negative_checkable_impl {
    ($($t:ty),*) => {
    $(
    	impl NonNegativeCheckable for $t {
    		fn is_non_negative(&self) -> bool {
    			*self >= (0 as Self)
    		}
    	}
    )*
    };
}

non_negative_checkable_impl!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);
