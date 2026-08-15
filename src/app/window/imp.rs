use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::SystemTime;

use crate::utils::gettext;
use gio::Settings;
use glib_macros::clone;
use gtk::gio::MenuItem;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, ApplicationWindow, PopoverMenu};
use std::sync::OnceLock;

use crate::app::utils::set_click_pass_through;
use crate::config::{Align, LyricDisplayMode};
use crate::sync::{OsImp, OS};

#[derive(Default)]
pub struct Window {
    pub settings: OnceLock<Settings>,

    pub clickthrough: Cell<bool>,
    pub cache_lyrics: Cell<bool>,

    pub lyric_align: Cell<Align>,
    pub lyric_display_mode: Cell<LyricDisplayMode>,
    pub above_font_size: Cell<i32>,
    pub below_font_size: Cell<i32>,
    pub above_color: RefCell<String>,
    pub below_color: RefCell<String>,
    pub glow_color: RefCell<String>,
    pub show_default_text_on_idle: Cell<bool>,
    pub show_lyric_on_pause: Cell<bool>,

    pub lyric_start: Cell<Option<SystemTime>>,
    pub lyric_offset_ms: Cell<i64>,
    pub length_toleration_ms: Cell<u128>,

    // widgets
    pub headerbar: gtk::HeaderBar,
    pub menubutton: gtk::MenuButton,
    pub menu: gio::Menu,
    pub player_menu: gio::Menu,
    pub display_mode_menu: gio::Menu,
    pub align_mode_menu: gio::Menu,
    pub font_size_menu: gio::Menu,
    #[cfg(feature = "import-lyric")]
    pub import_lyric_menu: gio::Menu,
    #[cfg(feature = "export-lyric")]
    pub export_lyric_menu: gio::Menu,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "GtkAppWindowSaveState";
    type Type = super::Window;
    type ParentType = ApplicationWindow;
}
impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
        // Load latest window state
        let obj = self.obj();
        obj.setup_settings();
        // set titlebar before loading state: whether to show it is a state
        obj.set_titlebar(Some(&self.headerbar));
        obj.load_window_state();

        self.headerbar.set_decoration_layout(Some("menu:close"));
        self.menubutton.set_icon_name("open-menu-symbolic");

        let hide_decoration = MenuItem::new(
            Some(&gettext("Hide Decoration")),
            Some("win.switch-decoration"),
        );
        let reload_theme = MenuItem::new(Some(&gettext("Reload theme")), Some("app.reload-theme"));
        let search_lyric = MenuItem::new(Some(&gettext("Search lyric")), Some("app.search-lyric"));
        let refetch_lyric =
            MenuItem::new(Some(&gettext("Refetch lyric")), Some("app.refetch-lyric"));
        let remove_lyric = MenuItem::new(
            Some(&if self.cache_lyrics.get() {
                gettext("Remove lyric")
            } else {
                gettext("Remove lyric forever")
            }),
            Some("app.remove-lyric"),
        );

        let popover = PopoverMenu::builder()
            .accessible_role(gtk::AccessibleRole::MenuItemRadio)
            .build();

        let ui_section = gio::Menu::default();
        let passthrough_item = gio::MenuItem::new(None, None);
        passthrough_item.set_attribute_value("custom", Some(&"passthrough-control".to_variant()));
        ui_section.append_item(&passthrough_item);

        let appearance_item = gio::MenuItem::new(None, None);
        appearance_item.set_attribute_value("custom", Some(&"appearance-controls".to_variant()));
        ui_section.append_item(&appearance_item);

        ui_section.append_submenu(
            Some(&gettext("Lyric Display Mode")),
            &self.display_mode_menu,
        );
        ui_section.append_submenu(
            Some(&gettext("Lyric Alignment")), //
            &self.align_mode_menu,
        );
        for item in [&hide_decoration, &reload_theme] {
            ui_section.append_item(item);
        }

        self.menu.append_section(None, &ui_section);

        let play_section = gio::Menu::default();
        play_section.append_submenu(
            Some(&gettext("Select Player")), //
            &self.player_menu,
        );

        #[cfg(feature = "import-lyric")]
        {
            self.import_lyric_menu.append(
                Some(&gettext("Original Lyric")),
                Some("app.import-lyric(true)"),
            );
            self.import_lyric_menu.append(
                Some(&gettext("Translated Lyric")),
                Some("app.import-lyric(false)"),
            );
            play_section.append_submenu(Some(&gettext("Import Lyric")), &self.import_lyric_menu);
        }
        #[cfg(feature = "export-lyric")]
        {
            self.export_lyric_menu.append(
                Some(&gettext("Original Lyric")),
                Some("app.export-lyric(true)"),
            );
            self.export_lyric_menu.append(
                Some(&gettext("Translated Lyric")),
                Some("app.export-lyric(false)"),
            );
            play_section.append_submenu(Some(&gettext("Export Lyric")), &self.export_lyric_menu);
        }

        for item in [&search_lyric, &remove_lyric, &refetch_lyric] {
            play_section.append_item(item);
        }

        self.menu.append_section(None, &play_section);

        popover.set_menu_model(Some(&self.menu));

        let appearance_controls = gtk::Box::new(gtk::Orientation::Vertical, 8);
        appearance_controls.set_margin_top(10);
        appearance_controls.set_margin_bottom(10);
        appearance_controls.set_margin_start(12);
        appearance_controls.set_margin_end(12);
        appearance_controls.set_width_request(320);

        let passthrough_controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        passthrough_controls.set_margin_top(10);
        passthrough_controls.set_margin_bottom(4);
        passthrough_controls.set_margin_start(12);
        passthrough_controls.set_margin_end(12);
        passthrough_controls.set_width_request(320);
        let passthrough_label = gtk::Label::new(Some("Toggle Passthrough"));
        passthrough_label.set_halign(gtk::Align::Start);
        passthrough_label.set_hexpand(true);
        passthrough_label.add_css_class("heading");
        let passthrough_switch = gtk::Switch::builder()
            .active(self.clickthrough.get())
            .build();
        passthrough_controls.append(&passthrough_label);
        passthrough_controls.append(&passthrough_switch);
        let window = self.obj();
        let window_weak = window.downgrade();
        passthrough_switch.connect_state_set(move |_, _| {
            if let Some(window) = window_weak.upgrade() {
                ActionGroupExt::activate_action(&window, "switch-passthrough", None);
            }
            Propagation::Proceed
        });
        popover.add_child(&passthrough_controls, "passthrough-control");

        let appearance_title = gtk::Label::new(Some("Appearance"));
        appearance_title.set_halign(gtk::Align::Start);
        appearance_title.add_css_class("heading");
        appearance_controls.append(&appearance_title);

        let above_adjustment =
            gtk::Adjustment::new(self.above_font_size.get() as f64, 14.0, 72.0, 1.0, 4.0, 0.0);
        let below_adjustment =
            gtk::Adjustment::new(self.below_font_size.get() as f64, 12.0, 64.0, 1.0, 4.0, 0.0);
        let above_row = font_size_row("Current lyric", &above_adjustment);
        let below_row = font_size_row("Secondary lyric", &below_adjustment);
        appearance_controls.append(&above_row);
        appearance_controls.append(&below_row);

        let color_title = gtk::Label::new(Some("Colors"));
        color_title.set_halign(gtk::Align::Start);
        color_title.add_css_class("heading");
        appearance_controls.append(&color_title);

        let (
            glow_color_group,
            glow_hue,
            glow_saturation,
            glow_brightness,
            glow_preview,
            glow_preview_color,
        ) = hsb_color_group("Glow", &self.glow_color.borrow());
        let (
            above_color_group,
            above_hue,
            above_saturation,
            above_brightness,
            above_preview,
            above_preview_color,
        ) = hsb_color_group("Current lyric", &self.above_color.borrow());
        let (
            below_color_group,
            below_hue,
            below_saturation,
            below_brightness,
            below_preview,
            below_preview_color,
        ) = hsb_color_group("Next lyric", &self.below_color.borrow());
        appearance_controls.append(&glow_color_group);
        appearance_controls.append(&above_color_group);
        appearance_controls.append(&below_color_group);

        let window = self.obj();
        above_adjustment.connect_value_changed(clone!(
            #[weak]
            window,
            move |adjustment| {
                window
                    .imp()
                    .above_font_size
                    .set(adjustment.value().round() as i32);
                crate::app::utils::apply_appearance(&window);
                let _ = window.save_window_state();
            }
        ));
        let window = self.obj();
        below_adjustment.connect_value_changed(clone!(
            #[weak]
            window,
            move |adjustment| {
                window
                    .imp()
                    .below_font_size
                    .set(adjustment.value().round() as i32);
                crate::app::utils::apply_appearance(&window);
                let _ = window.save_window_state();
            }
        ));
        let window = self.obj();
        bind_hsb_controls(
            &window,
            ColorTarget::Above,
            &above_hue,
            &above_saturation,
            &above_brightness,
            &above_preview,
            &above_preview_color,
        );
        bind_hsb_controls(
            &window,
            ColorTarget::Below,
            &below_hue,
            &below_saturation,
            &below_brightness,
            &below_preview,
            &below_preview_color,
        );
        bind_hsb_controls(
            &window,
            ColorTarget::Glow,
            &glow_hue,
            &glow_saturation,
            &glow_brightness,
            &glow_preview,
            &glow_preview_color,
        );
        popover.add_child(&appearance_controls, "appearance-controls");

        let player_menu = &self.player_menu;
        popover.connect_visible_submenu_notify(clone!(
            #[weak]
            player_menu,
            move |sub| {
                if Some(&*gettext("Select Player")) != sub.visible_submenu().as_deref() {
                    return;
                }
                player_menu.remove_all();

                let section = gio::Menu::new();
                let players = OS::list_players();
                if !players.is_empty() {
                    let disconnect =
                        MenuItem::new(Some(&gettext("Disconnect")), Some("app.disconnect"));
                    player_menu.append_item(&disconnect);
                }

                for player in players {
                    let item = MenuItem::new(Some(&player.player_name), None);
                    item.set_action_and_target_value(
                        Some("app.connect"),
                        Some(&ToVariant::to_variant(&player.inner_id)),
                    );
                    section.append_item(&item);
                }
                player_menu.append_section(None, &section);
            }
        ));
        self.menubutton.set_popover(Some(&popover));

        for display_mode in <LyricDisplayMode as strum::IntoEnumIterator>::iter() {
            let display_mode_str = display_mode.to_string();
            let item = MenuItem::new(Some(&gettext(&display_mode_str).replace("_", "__")), None);
            item.set_action_and_target_value(
                Some("win.set-display-mode"),
                Some(&display_mode_str.to_variant()),
            );
            self.display_mode_menu.append_item(&item);
        }

        for lyric_align in <Align as strum::IntoEnumIterator>::iter() {
            let lyric_align_str = lyric_align.to_string();
            let item = MenuItem::new(Some(&gettext(&lyric_align_str)), None);
            item.set_action_and_target_value(
                Some("win.set-lyric-align"),
                Some(&lyric_align_str.to_variant()),
            );
            self.align_mode_menu.append_item(&item);
        }

        self.headerbar.pack_end(&self.menubutton)
    }
}

fn font_size_row(label: &str, adjustment: &gtk::Adjustment) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some(label));
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(16);
    let scale = gtk::Scale::new(gtk::Orientation::Horizontal, Some(adjustment));
    scale.set_draw_value(false);
    scale.set_hexpand(true);
    let spin = gtk::SpinButton::new(Some(adjustment), 1.0, 0);
    spin.set_numeric(true);
    spin.set_width_chars(4);
    row.append(&label);
    row.append(&scale);
    row.append(&spin);
    row
}

#[allow(dead_code)]
fn color_row_removed(label: &str, color: &str) -> (gtk::Box, gtk::Entry) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some(label));
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);
    let entry = gtk::Entry::new();
    entry.set_text(color);
    entry.set_width_chars(18);
    entry.set_placeholder_text(Some("#RRGGBB or rgba(...)"));
    row.append(&label);
    row.append(&entry);
    (row, entry)
}
#[derive(Clone, Copy)]
enum ColorTarget {
    Above,
    Below,
    Glow,
}

fn hsb_color_group(
    label: &str,
    color: &str,
) -> (
    gtk::Box,
    gtk::Adjustment,
    gtk::Adjustment,
    gtk::Adjustment,
    gtk::DrawingArea,
    Rc<RefCell<gtk::gdk::RGBA>>,
) {
    let rgba = gtk::gdk::RGBA::parse(color).unwrap_or(gtk::gdk::RGBA::WHITE);
    let (hue, saturation, brightness) = rgba_to_hsb(&rgba);
    let hue = gtk::Adjustment::new(hue, 0.0, 360.0, 1.0, 30.0, 0.0);
    let saturation = gtk::Adjustment::new(saturation, 0.0, 100.0, 1.0, 10.0, 0.0);
    let brightness = gtk::Adjustment::new(brightness, 0.0, 100.0, 1.0, 10.0, 0.0);

    let preview_color = Rc::new(RefCell::new(rgba));
    let preview = gtk::DrawingArea::new();
    preview.set_content_width(28);
    preview.set_content_height(18);
    let preview_color_for_draw = preview_color.clone();
    preview.set_draw_func(move |_, cr, width, height| {
        let color = preview_color_for_draw.borrow();
        cr.set_source_rgba(
            color.red() as f64,
            color.green() as f64,
            color.blue() as f64,
            color.alpha() as f64,
        );
        cr.rectangle(1.0, 1.0, (width - 2) as f64, (height - 2) as f64);
        let _ = cr.fill();
        cr.set_source_rgba(0.45, 0.45, 0.45, 0.8);
        cr.set_line_width(1.0);
        cr.rectangle(0.5, 0.5, (width - 1) as f64, (height - 1) as f64);
        let _ = cr.stroke();
    });

    let group = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let title = gtk::Label::new(Some(label));
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);
    title.add_css_class("dim-label");
    header.append(&title);
    header.append(&preview);
    group.append(&header);
    group.append(&hsb_slider_row("H", &hue));
    group.append(&hsb_slider_row("S", &saturation));
    group.append(&hsb_slider_row("B", &brightness));
    (group, hue, saturation, brightness, preview, preview_color)
}

fn hsb_slider_row(label: &str, adjustment: &gtk::Adjustment) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let label = gtk::Label::new(Some(label));
    label.set_width_chars(2);
    let scale = gtk::Scale::new(gtk::Orientation::Horizontal, Some(adjustment));
    scale.set_draw_value(true);
    scale.set_digits(0);
    scale.set_hexpand(true);
    row.append(&label);
    row.append(&scale);
    row
}

fn rgba_to_hsb(color: &gtk::gdk::RGBA) -> (f64, f64, f64) {
    let red = color.red() as f64;
    let green = color.green() as f64;
    let blue = color.blue() as f64;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == red {
        60.0 * (((green - blue) / delta) % 6.0)
    } else if max == green {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    };
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (hue, saturation * 100.0, max * 100.0)
}

fn hsb_to_css(hue: f64, saturation: f64, brightness: f64, alpha: f64) -> String {
    let saturation = (saturation / 100.0).clamp(0.0, 1.0);
    let brightness = (brightness / 100.0).clamp(0.0, 1.0);
    let chroma = brightness * saturation;
    let sector = hue.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - ((sector % 2.0) - 1.0).abs());
    let (red, green, blue) = match sector {
        s if s < 1.0 => (chroma, x, 0.0),
        s if s < 2.0 => (x, chroma, 0.0),
        s if s < 3.0 => (0.0, chroma, x),
        s if s < 4.0 => (0.0, x, chroma),
        s if s < 5.0 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let match_value = brightness - chroma;
    let red = ((red + match_value) * 255.0).round() as u8;
    let green = ((green + match_value) * 255.0).round() as u8;
    let blue = ((blue + match_value) * 255.0).round() as u8;
    format!("rgba({red}, {green}, {blue}, {:.3})", alpha.clamp(0.0, 1.0))
}

fn bind_hsb_controls(
    window: &super::Window,
    target: ColorTarget,
    hue: &gtk::Adjustment,
    saturation: &gtk::Adjustment,
    brightness: &gtk::Adjustment,
    preview: &gtk::DrawingArea,
    preview_color: &Rc<RefCell<gtk::gdk::RGBA>>,
) {
    let preview_h = preview.clone();
    let preview_color_h = preview_color.clone();
    let h = hue.clone();
    let s = saturation.clone();
    let b = brightness.clone();
    hue.connect_value_changed(clone!(
        #[weak]
        window,
        move |_| update_hsb_color(&window, target, &h, &s, &b, &preview_h, &preview_color_h),
    ));
    let preview_s = preview.clone();
    let preview_color_s = preview_color.clone();
    let h = hue.clone();
    let s = saturation.clone();
    let b = brightness.clone();
    saturation.connect_value_changed(clone!(
        #[weak]
        window,
        move |_| update_hsb_color(&window, target, &h, &s, &b, &preview_s, &preview_color_s),
    ));
    let preview_b = preview.clone();
    let preview_color_b = preview_color.clone();
    let h = hue.clone();
    let s = saturation.clone();
    let b = brightness.clone();
    brightness.connect_value_changed(clone!(
        #[weak]
        window,
        move |_| update_hsb_color(&window, target, &h, &s, &b, &preview_b, &preview_color_b),
    ));
}

fn update_hsb_color(
    window: &super::Window,
    target: ColorTarget,
    hue: &gtk::Adjustment,
    saturation: &gtk::Adjustment,
    brightness: &gtk::Adjustment,
    preview: &gtk::DrawingArea,
    preview_color: &Rc<RefCell<gtk::gdk::RGBA>>,
) {
    let current = match target {
        ColorTarget::Above => window.imp().above_color.borrow().clone(),
        ColorTarget::Below => window.imp().below_color.borrow().clone(),
        ColorTarget::Glow => window.imp().glow_color.borrow().clone(),
    };
    let alpha = gtk::gdk::RGBA::parse(&current)
        .map(|color| color.alpha() as f64)
        .unwrap_or(1.0);
    let color = hsb_to_css(hue.value(), saturation.value(), brightness.value(), alpha);
    match target {
        ColorTarget::Above => *window.imp().above_color.borrow_mut() = color.clone(),
        ColorTarget::Below => *window.imp().below_color.borrow_mut() = color.clone(),
        ColorTarget::Glow => *window.imp().glow_color.borrow_mut() = color.clone(),
    }
    if let Ok(rgba) = gtk::gdk::RGBA::parse(&color) {
        *preview_color.borrow_mut() = rgba;
        preview.set_tooltip_text(Some(&color));
        preview.queue_draw();
    }
    crate::app::utils::apply_appearance(window);
    let _ = window.save_window_state();
}

impl WidgetImpl for Window {
    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);
        let clickthrough = self.clickthrough.get();
        set_click_pass_through(&self.obj(), clickthrough);
    }
}
impl WindowImpl for Window {
    // Save window state right before the window will be closed
    fn close_request(&self) -> Propagation {
        // Save window size
        self.obj()
            .save_window_state()
            .expect("Failed to save window state");

        // Don't invoke the default handler
        Propagation::Proceed
    }
}
impl ApplicationWindowImpl for Window {}
