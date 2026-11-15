use gtk4::cairo;
use gtk4::gdk::Key;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, DrawingArea,
    EventControllerKey, Label, Orientation, Overlay,
};
use std::cell::Cell;
use std::f64::consts::PI;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

const APP_ID: &str = "dev.koresent.SnapHud";

const HUD_CSS: &str = r#"
window.background.snap-hud-window {
    background-color: #282828;
    background-image: none;
    color: #ebdbb2;
}

.hud-pill {
    background-color: #282828;
    padding: 8px 10px;
}

.mode-btn {
    background-color: transparent;
    background-image: none;
    border: 1px solid transparent;
    border-radius: 11px;
    padding: 9px 14px;
    color: #ebdbb2;
    box-shadow: none;
    transition: background-color 110ms ease-out, border-color 110ms ease-out;
}

.mode-btn:hover,
.mode-btn:focus {
    background-color: #3c3836;
    border-color: #504945;
    outline: none;
}

.mode-btn:active {
    background-color: #504945;
}

.mode-title {
    color: #ebdbb2;
    font-weight: 600;
    font-size: 13px;
}

.mode-key {
    background-color: #3c3836;
    color: #fabd2f;
    border: 1px solid #504945;
    border-radius: 6px;
    padding: 1px 6px;
    font-family: "Maple Mono NF", monospace;
    font-size: 11px;
    font-weight: 700;
}

.mode-btn:hover .mode-key,
.mode-btn:focus .mode-key {
    border-color: #665c54;
}

.pretty-dot {
    background-color: #fabd2f;
    border-radius: 999px;
    min-width: 7px;
    min-height: 7px;
}
"#;

#[derive(Clone, Copy)]
enum ModeIcon {
    Area,
    Window,
    Screen,
}

struct ModeSpec {
    id: &'static str,
    key_hint: &'static str,
    icon: ModeIcon,
    title: &'static str,
}

const MODES: &[ModeSpec] = &[
    ModeSpec {
        id: "area",
        key_hint: "1",
        icon: ModeIcon::Area,
        title: "Area",
    },
    ModeSpec {
        id: "window",
        key_hint: "2",
        icon: ModeIcon::Window,
        title: "Window",
    },
    ModeSpec {
        id: "screen",
        key_hint: "3",
        icon: ModeIcon::Screen,
        title: "Screen",
    },
];

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(HUD_CSS);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn draw_rounded_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    cr.arc(x + r, y + h - r, r, PI / 2.0, PI);
    cr.arc(x + r, y + r, r, PI, 1.5 * PI);
    cr.close_path();
}

fn draw_mode_icon(cr: &cairo::Context, kind: ModeIcon) {
    cr.set_line_width(1.7);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);

    match kind {
        ModeIcon::Area => {
            cr.set_source_rgb(0.722, 0.733, 0.149);
            cr.set_dash(&[3.2, 2.8], 0.0);
            draw_rounded_rect(cr, 3.0, 3.0, 14.0, 14.0, 2.0);
            let _ = cr.stroke();
            cr.set_dash(&[], 0.0);

            cr.set_source_rgb(0.984, 0.741, 0.184);
            cr.move_to(17.0, 13.5);
            cr.line_to(17.0, 20.5);
            cr.move_to(13.5, 17.0);
            cr.line_to(20.5, 17.0);
            let _ = cr.stroke();
        }
        ModeIcon::Window => {
            cr.set_source_rgb(0.722, 0.733, 0.149);
            draw_rounded_rect(cr, 2.5, 3.5, 17.0, 15.0, 2.5);
            let _ = cr.stroke();

            cr.move_to(2.5, 7.8);
            cr.line_to(19.5, 7.8);
            let _ = cr.stroke();

            cr.set_source_rgb(0.984, 0.741, 0.184);
            for cx in [5.5, 8.2] {
                cr.arc(cx, 5.6, 0.85, 0.0, 2.0 * PI);
                let _ = cr.fill();
            }
        }
        ModeIcon::Screen => {
            cr.set_source_rgb(0.722, 0.733, 0.149);
            draw_rounded_rect(cr, 2.0, 3.0, 18.0, 12.5, 2.0);
            let _ = cr.stroke();

            cr.set_source_rgb(0.922, 0.859, 0.698);
            cr.move_to(11.0, 15.5);
            cr.line_to(11.0, 18.8);
            cr.move_to(7.0, 18.8);
            cr.line_to(15.0, 18.8);
            let _ = cr.stroke();
        }
    }
}

pub fn mode_from_key(key: Key) -> Option<Option<&'static str>> {
    match key {
        Key::Escape | Key::q | Key::Q | Key::Cyrillic_shorti | Key::Cyrillic_SHORTI => {
            Some(None)
        }
        Key::_1 | Key::KP_1 | Key::a | Key::A | Key::Cyrillic_ef | Key::Cyrillic_EF => {
            Some(Some("area"))
        }
        Key::_2 | Key::KP_2 | Key::w | Key::W | Key::Cyrillic_tse | Key::Cyrillic_TSE => {
            Some(Some("window"))
        }
        Key::_3
        | Key::KP_3
        | Key::s
        | Key::S
        | Key::Cyrillic_yeru
        | Key::Cyrillic_YERU => Some(Some("screen")),
        _ => None,
    }
}

fn finish_with_mode(
    selection: &Rc<Cell<Option<&'static str>>>,
    window: &ApplicationWindow,
    app: &Application,
    mode: Option<&'static str>,
) {
    selection.set(mode);
    window.close();
    app.quit();
}

fn build_mode_button(
    spec: &'static ModeSpec,
    selection: Rc<Cell<Option<&'static str>>>,
    window: ApplicationWindow,
    app: Application,
) -> Button {
    let col = GtkBox::new(Orientation::Vertical, 6);
    col.set_halign(Align::Center);
    col.set_valign(Align::Center);

    let top = GtkBox::new(Orientation::Horizontal, 8);
    top.set_halign(Align::Center);

    let icon = DrawingArea::builder()
        .content_width(22)
        .content_height(22)
        .build();
    let kind = spec.icon;
    icon.set_draw_func(move |_, cr, _, _| draw_mode_icon(cr, kind));

    let key_badge = Label::new(Some(spec.key_hint));
    key_badge.add_css_class("mode-key");

    top.append(&icon);
    top.append(&key_badge);

    let title = Label::new(Some(spec.title));
    title.add_css_class("mode-title");

    col.append(&top);
    col.append(&title);

    let btn = Button::new();
    btn.add_css_class("mode-btn");
    btn.set_child(Some(&col));
    btn.connect_clicked(move |_| {
        finish_with_mode(&selection, &window, &app, Some(spec.id));
    });
    btn
}

fn main() {
    // Use Cairo GSK renderer to avoid NVIDIA GL/Vulkan cold-start latency on a small popover
    if std::env::var_os("GSK_RENDERER").is_none() {
        // SAFETY: Executed at startup before GTK spawns any background threads.
        unsafe {
            std::env::set_var("GSK_RENDERER", "cairo");
        }
    }

    let pretty = std::env::args().any(|a| a == "--pretty");
    let selection: Rc<Cell<Option<&'static str>>> = Rc::new(Cell::new(None));

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    let sel_activate = Rc::clone(&selection);
    app.connect_activate(move |app| {
        if let Some(window) = app.windows().first() {
            window.present();
            return;
        }

        load_css();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Screenshot")
            .decorated(false)
            .resizable(false)
            .build();
        window.add_css_class("snap-hud-window");

        let pill = GtkBox::new(Orientation::Horizontal, 6);
        pill.add_css_class("hud-pill");

        let mut first_btn: Option<Button> = None;
        for spec in MODES {
            let btn = build_mode_button(
                spec,
                Rc::clone(&sel_activate),
                window.clone(),
                app.clone(),
            );
            if first_btn.is_none() {
                first_btn = Some(btn.clone());
            }
            pill.append(&btn);
        }

        let root = Overlay::new();
        root.set_child(Some(&pill));

        if pretty {
            let dot = GtkBox::new(Orientation::Horizontal, 0);
            dot.add_css_class("pretty-dot");
            dot.set_halign(Align::End);
            dot.set_valign(Align::Start);
            dot.set_margin_top(9);
            dot.set_margin_end(9);
            dot.set_can_target(false);
            dot.set_tooltip_text(Some("Rounded corners & drop shadow"));
            root.add_overlay(&dot);
        }

        let key_ctrl = EventControllerKey::new();
        let sel_key = Rc::clone(&sel_activate);
        let win_key = window.clone();
        let app_key = app.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, _| {
            match mode_from_key(key) {
                Some(maybe_mode) => {
                    finish_with_mode(&sel_key, &win_key, &app_key, maybe_mode);
                    gtk4::glib::Propagation::Stop
                }
                None => gtk4::glib::Propagation::Proceed,
            }
        });
        window.add_controller(key_ctrl);

        window.set_child(Some(&root));
        window.present();
        if let Some(btn) = first_btn {
            btn.grab_focus();
        }
    });

    app.run_with_args::<&str>(&[]);

    let Some(mode) = selection.get() else {
        return;
    };

    // Wait for Umbriel's close fade before capturing the full monitor
    if mode == "screen" {
        std::thread::sleep(Duration::from_millis(260));
    } else {
        std::thread::sleep(Duration::from_millis(60));
    }

    let capture_bin = std::env::var("HOME")
        .map(|h| format!("{h}/.local/bin/snap-capture"))
        .unwrap_or_else(|_| "snap-capture".to_string());

    let mut cmd = Command::new(&capture_bin);
    cmd.arg(mode);
    if pretty {
        cmd.arg("--pretty");
    }
    let err = cmd.exec();
    eprintln!("snap-hud: failed to exec {capture_bin}: {err}");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::{MODES, mode_from_key};
    use gtk4::gdk::Key;

    #[test]
    fn modes_list_contains_expected_definitions() {
        assert_eq!(MODES.len(), 3);
        assert_eq!(MODES[0].id, "area");
        assert_eq!(MODES[1].id, "window");
        assert_eq!(MODES[2].id, "screen");
    }

    #[test]
    fn maps_keys_to_modes_including_cyrillic() {
        // Area shortcuts
        assert_eq!(mode_from_key(Key::_1), Some(Some("area")));
        assert_eq!(mode_from_key(Key::a), Some(Some("area")));
        assert_eq!(mode_from_key(Key::Cyrillic_ef), Some(Some("area")));

        // Window shortcuts
        assert_eq!(mode_from_key(Key::_2), Some(Some("window")));
        assert_eq!(mode_from_key(Key::w), Some(Some("window")));
        assert_eq!(mode_from_key(Key::Cyrillic_tse), Some(Some("window")));

        // Screen shortcuts
        assert_eq!(mode_from_key(Key::_3), Some(Some("screen")));
        assert_eq!(mode_from_key(Key::s), Some(Some("screen")));
        assert_eq!(mode_from_key(Key::Cyrillic_yeru), Some(Some("screen")));

        // Exit shortcuts
        assert_eq!(mode_from_key(Key::Escape), Some(None));
        assert_eq!(mode_from_key(Key::q), Some(None));
        assert_eq!(mode_from_key(Key::Cyrillic_shorti), Some(None));

        // Unbound key
        assert_eq!(mode_from_key(Key::z), None);
    }
}
