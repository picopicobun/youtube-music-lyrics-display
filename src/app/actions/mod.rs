#[cfg(feature = "action-event")]
mod event;

#[cfg(feature = "action-event")]
pub use event::{init_ui_action_channel, UIAction, UI_ACTION};

use crate::app::utils::set_click_pass_through;
use crate::app::Window;

use crate::config::Align;
use crate::log::error;
use crate::utils::bind_shortcut;

use glib_macros::clone;
use gtk::gio::SimpleAction;
use gtk::glib::{self, Variant, VariantTy};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::Application;

use super::{get_label, set_lyric_align};

pub fn register_switch_decoration(wind: &Window, trigger: &str) {
    let action = SimpleAction::new("switch-decoration", None);
    action.connect_activate(clone!(
        #[weak]
        wind,
        move |_, _| {
            wind.set_decorated(!wind.is_decorated());
        }
    ));
    wind.add_action(&action);

    bind_shortcut("win.switch-decoration", wind, trigger);
}

pub fn register_reload_theme(app: &Application, wind: &Window, trigger: &str) {
    let action = SimpleAction::new("reload-theme", None);
    action.connect_activate(move |_, _| {
        crate::THEME_PATH.with_borrow(|theme_path| {
            if let Ok(style) = std::fs::read_to_string(theme_path) {
                crate::app::utils::merge_css(&style);
            }
        });
    });
    app.add_action(&action);
    bind_shortcut("app.reload-theme", wind, trigger);
}

pub fn register_switch_passthrough(wind: &Window, trigger: &str) {
    let action = SimpleAction::new("switch-passthrough", None);
    action.connect_activate(clone!(
        #[weak]
        wind,
        move |_, _| {
            let clickthrough = !wind.imp().clickthrough.get();
            wind.imp().clickthrough.set(clickthrough);
            set_click_pass_through(&wind, clickthrough);
            wind.present();
        }
    ));
    wind.add_action(&action);

    bind_shortcut("win.switch-passthrough", wind, trigger);
}

pub fn register_font_size(wind: &Window) {
    let dialog = SimpleAction::new("font-size-dialog", None);
    dialog.connect_activate(clone!(
        #[weak]
        wind,
        move |_, _| show_font_size_dialog(&wind),
    ));
    wind.add_action(&dialog);

    let increase = SimpleAction::new("increase-font-size", None);
    increase.connect_activate(clone!(
        #[weak]
        wind,
        move |_, _| adjust_font_size(&wind, 2),
    ));
    wind.add_action(&increase);

    let decrease = SimpleAction::new("decrease-font-size", None);
    decrease.connect_activate(clone!(
        #[weak]
        wind,
        move |_, _| adjust_font_size(&wind, -2),
    ));
    wind.add_action(&decrease);

    let reset = SimpleAction::new("reset-font-size", None);
    reset.connect_activate(clone!(
        #[weak]
        wind,
        move |_, _| {
            wind.imp().above_font_size.set(28);
            wind.imp().below_font_size.set(24);
            super::utils::apply_font_sizes(&wind);
        }
    ));
    wind.add_action(&reset);
}

fn show_font_size_dialog(wind: &Window) {
    let dialog = gtk::Dialog::builder()
        .title("Font Size")
        .transient_for(wind)
        .modal(true)
        .build();

    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let above = gtk::SpinButton::with_range(14.0, 72.0, 1.0);
    above.set_value(wind.imp().above_font_size.get() as f64);
    above.set_numeric(true);
    let below = gtk::SpinButton::with_range(12.0, 64.0, 1.0);
    below.set_value(wind.imp().below_font_size.get() as f64);
    below.set_numeric(true);

    grid.attach(&gtk::Label::new(Some("Current lyric")), 0, 0, 1, 1);
    grid.attach(&above, 1, 0, 1, 1);
    grid.attach(&gtk::Label::new(Some("Secondary lyric")), 0, 1, 1, 1);
    grid.attach(&below, 1, 1, 1, 1);
    dialog.content_area().append(&grid);
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Apply", gtk::ResponseType::Accept);
    dialog.set_default_response(gtk::ResponseType::Accept);

    let wind = wind.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            wind.imp().above_font_size.set(above.value_as_int());
            wind.imp().below_font_size.set(below.value_as_int());
            super::utils::apply_font_sizes(&wind);
        }
        dialog.close();
    });
    dialog.present();
}

fn adjust_font_size(wind: &Window, delta: i32) {
    let above = (wind.imp().above_font_size.get() + delta).clamp(14, 72);
    let below = (wind.imp().below_font_size.get() + delta).clamp(12, 64);
    wind.imp().above_font_size.set(above);
    wind.imp().below_font_size.set(below);
    super::utils::apply_font_sizes(wind);
}

pub fn register_set_display_mode(wind: &Window) {
    let action = SimpleAction::new("set-display-mode", Some(VariantTy::STRING));
    action.connect_activate(clone!(
        #[weak]
        wind,
        move |_, display_mode| {
            let Some(display_mode) = display_mode.and_then(|d| d.str()) else {
                return;
            };
            let Ok(display_mode) = display_mode.parse() else {
                error!("unknown display_mode: {display_mode}");
                return;
            };
            wind.imp().lyric_display_mode.set(display_mode);
        }
    ));
    wind.add_action(&action);
}

pub fn register_set_lyric_align(wind: &Window) {
    let action = SimpleAction::new("set-lyric-align", Some(VariantTy::STRING));
    action.connect_activate(clone!(
        #[weak]
        wind,
        move |_, lyric_align| {
            let Some(align) = lyric_align.and_then(|d| d.str()) else {
                return;
            };
            let Ok(align): Result<Align, _> = align.parse() else {
                error!("unknown lyric alignment: {align}");
                return;
            };
            set_lyric_align(&wind, align);
        }
    ));
    wind.add_action(&action);
}

pub fn register_set_label(wind: &Window) {
    let action = SimpleAction::new("set-label", Some(VariantTy::STRING_ARRAY));
    action.connect_activate(clone!(
        #[weak]
        wind,
        move |_, args| {
            let Some((position, text)) = args.and_then(extract_str_array) else {
                return;
            };
            get_label(&wind, position).set_label(text);
        }
    ));
    wind.add_action(&action);
}

fn extract_str_array(variant: &Variant) -> Option<(&str, &str)> {
    let mut iter = variant.array_iter_str().ok()?;
    let position = iter.next()?;

    if !["above", "below"].contains(&position) {
        return None;
    }

    let text = iter.next()?;

    Some((position, text))
}
