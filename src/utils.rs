use std::{fmt::Display, str::FromStr};

use egui::Ui;

pub fn float_input<T>(ui: &mut Ui, text: &mut String, value: &mut T) where T: FromStr + Display {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        //ensure entered characters are valid in a float
        if new_text.find(|char: char| {
            !char.is_numeric() && char != '.' && char != '-'
        }) == None {
            *text = new_text;
        }
    }

    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<T>() {
            *value = new_value;
        }
        *text = value.to_string();
    }
}

pub fn signed_int_input<T>(ui: &mut Ui, text: &mut String, value: &mut T) where T: FromStr + Display {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        //ensure entered characters are valid in a signed integer
        if new_text.find(|char: char| {
            !char.is_numeric() && char != '-'
        }) == None {
            *text = new_text;
        }
    }

    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<T>() {
            *value = new_value;
        }
        *text = value.to_string();
    }
}

pub fn unsigned_int_input<T>(ui: &mut Ui, text: &mut String, value: &mut T) where T: FromStr + Display {
    let mut new_text = text.clone();
    let response = ui.text_edit_singleline(&mut new_text);
    if response.changed() {
        //ensure entered characters are valid in an unsigned integer
        if new_text.find(|char: char| { !char.is_numeric() }) == None {
            *text = new_text;
        }
    }

    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(new_value) = text.parse::<T>() {
            *value = new_value;
        }
        *text = value.to_string();
    }
}
