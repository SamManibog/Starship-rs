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

pub fn pitch_or_value_input<T>(ui: &mut Ui, text: &mut String, value: &mut PitchOrValue<T>) where T: FromStr + Display {
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

pub fn float_input<T>(ui: &mut Ui, text: &mut String, value: &mut T) where T: FromStr + Display {
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
