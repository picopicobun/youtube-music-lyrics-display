use crate::EXCLUDED_REGEXES;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::Label;
use std::cell::RefCell;

use super::window;

#[cfg(target_os = "windows")]
pub(super) fn set_click_pass_through(window: &window::Window, enabled: bool) {
    use std::ffi::c_void;

    fn set_window_click_through(hwnd: *mut c_void, enabled: bool) {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE,
        };
        let hwnd = HWND(hwnd as _);

        const WS_EX_TRANSPARENT: isize = 0x00000020;
        const WS_EX_LAYERED: isize = 0x00080000;
        unsafe {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if enabled {
                SetWindowLongPtrW(
                    hwnd,
                    GWL_EXSTYLE,
                    ex_style | WS_EX_TRANSPARENT | WS_EX_LAYERED,
                );
            } else {
                SetWindowLongPtrW(
                    hwnd,
                    GWL_EXSTYLE,
                    ex_style & !WS_EX_TRANSPARENT & !WS_EX_LAYERED,
                );
            }
        }
    }

    let Some(surface) = window.surface().and_downcast::<gdk4_win32::Win32Surface>() else {
        return;
    };

    let handle = surface.handle().0;

    set_window_click_through(handle, enabled);
}

#[cfg(target_os = "windows")]
pub(super) fn set_window_topmost(window: &window::Window) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };

    let Some(surface) = window.surface().and_downcast::<gdk4_win32::Win32Surface>() else {
        return;
    };

    let hwnd = HWND(surface.handle().0 as _);
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub(super) fn set_window_topmost(_window: &window::Window) {}

#[cfg(not(target_os = "windows"))]
pub(super) fn set_click_pass_through(window: &window::Window, enabled: bool) {
    use gtk::cairo::{RectangleInt, Region};
    use gtk::subclass::prelude::*;

    let obj = window;
    let Some(surface) = obj.surface() else {
        return;
    };

    if enabled {
        if !window.is_decorated() {
            surface.set_input_region(Some(&Region::create_rectangle(&RectangleInt::new(
                0, 0, 0, 0,
            ))));
        } else {
            let headerbar = &window.imp().headerbar;
            let allocation = headerbar.allocation();

            surface.set_input_region(Some(&Region::create_rectangle(&RectangleInt::new(
                allocation.x(),
                allocation.y(),
                allocation.width(),
                allocation.height(),
            ))));
        }
    } else {
        surface.set_input_region(Some(&Region::create_rectangle(&RectangleInt::new(
            0,
            0,
            i32::MAX,
            i32::MAX,
        ))));
    }
}

/// set css style for waylyrics
/// As said in [GTK+ doc], gtk constructs style from the lower priority ones to the upper ones,
/// We set priority as `STYLE_PROVIDER_PRIORITY + 1` to override user theme
///
/// [GTK+ doc]: https://docs.gtk.org/gtk4/type_func.StyleContext.add_provider_for_display.html#parameters
pub fn merge_css(css: &str) {
    use gtk::gdk::Display as GdkDisplay;
    use gtk::CssProvider;
    use std::cell::RefCell;

    thread_local! {
        static LATEST_PROVIDER: RefCell<Option<CssProvider>> = const { RefCell::new(None) };
    }
    let css_provider = CssProvider::new();
    css_provider.load_from_data(css);
    let display = GdkDisplay::default().expect("Could not connect to a display.");
    LATEST_PROVIDER.with_borrow_mut(|provider| {
        if let Some(provider) = provider.take() {
            gtk::style_context_remove_provider_for_display(&display, &provider);
        }
    });

    gtk::style_context_add_provider_for_display(
        &display,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
    );
    LATEST_PROVIDER.with_borrow_mut(|provider| {
        *provider = Some(css_provider);
    });
}

thread_local! {
    static FONT_SIZE_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

pub fn apply_font_sizes(window: &window::Window) {
    apply_appearance(window);
}

pub fn apply_appearance(window: &window::Window) {
    use gtk::gdk::Display as GdkDisplay;

    let provider = gtk::CssProvider::new();
    let above = window.imp().above_font_size.get();
    let below = window.imp().below_font_size.get();
    let above_color = window.imp().above_color.borrow().clone();
    let below_color = window.imp().below_color.borrow().clone();
    let glow_color = window.imp().glow_color.borrow().clone();
    let below_glow_color = dim_color(&glow_color, 0.5);
    let css = format!(
        "label#above {{
            font-size: {above}px;
            color: {above_color};
            text-shadow:
                -1px -1px 0 rgba(0, 0, 0, 0.95),
                 1px -1px 0 rgba(0, 0, 0, 0.95),
                -1px  1px 0 rgba(0, 0, 0, 0.95),
                 1px  1px 0 rgba(0, 0, 0, 0.95),
                 0 0 5px {glow_color},
                 0 0 14px {glow_color};
        }}
        label#below {{
            font-size: {below}px;
            color: {below_color};
            text-shadow:
                -1px -1px 0 rgba(0, 0, 0, 0.90),
                 1px -1px 0 rgba(0, 0, 0, 0.90),
                -1px  1px 0 rgba(0, 0, 0, 0.90),
                 1px  1px 0 rgba(0, 0, 0, 0.90),
                 0 0 5px {below_glow_color},
                 0 0 12px {below_glow_color};
        }}"
    );
    provider.load_from_data(&css);

    let display = GdkDisplay::default().expect("Could not connect to a display.");
    FONT_SIZE_PROVIDER.with_borrow_mut(|previous| {
        if let Some(previous) = previous.take() {
            gtk::style_context_remove_provider_for_display(&display, &previous);
        }
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER + 2,
        );
        *previous = Some(provider);
    });
}

fn dim_color(color: &str, factor: f32) -> String {
    let Ok(color) = gtk::gdk::RGBA::parse(color) else {
        return color.to_string();
    };
    rgba_to_css(&color.with_alpha(color.alpha() * factor))
}

fn rgba_to_css(color: &gtk::gdk::RGBA) -> String {
    let red = (color.red().clamp(0.0, 1.0) * 255.0).round() as u8;
    let green = (color.green().clamp(0.0, 1.0) * 255.0).round() as u8;
    let blue = (color.blue().clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "rgba({red}, {green}, {blue}, {:.3})",
        color.alpha().clamp(0.0, 1.0)
    )
}

fn has_filtered_word(text: &str) -> bool {
    EXCLUDED_REGEXES.with_borrow(|regex_set| regex_set.is_match(text))
}

pub fn setup_label(label: &Label, hide_filtered_words: bool) {
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::Word);

    if hide_filtered_words {
        label.connect_label_notify(|label| {
            let text = label.label();
            let visible = !has_filtered_word(&text) && !text.is_empty();
            label.set_visible(visible);
        });
    } else {
        label.connect_label_notify(|label| {
            let visible = !label.label().is_empty();
            label.set_visible(visible);
        });
    }
}
