use gtk::gdk;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Settings};
use std::process::{Command, exit};

const APP_ID: &str = "com.example.FirstGtkApp";
const WINDOW_WIDTH: i32 = 300;
const CSS_DATA: &[u8] = include_bytes!("../styles/kanagawa.css");

// Define commands as constants to avoid string duplication
mod commands {
    pub const SHUTDOWN: &str = "systemctl poweroff";
    pub const REBOOT: &str = "systemctl reboot";
    pub const EXIT: &str = "swaymsg exit";
    pub const SUSPEND: &str = "systemctl suspend";
    pub const LOCK: &str = "swaylock \
        --screenshots \
        --clock \
        --indicator \
        --indicator-radius 100 \
        --indicator-thickness 7 \
        --effect-blur 7x5";
}

#[derive(Clone)]
struct PowerMenuApp {
    window: ApplicationWindow,
}

impl PowerMenuApp {
    fn new(app: &Application) -> Self {
        Self::load_css();
        Self::set_dark_theme();

        let window = Self::create_window(app);
        let app_instance = Self {
            window: window.clone(),
        };

        app_instance.setup_window_properties();
        app_instance.setup_event_handlers();
        app_instance.create_ui();

        app_instance.show();
        app_instance
    }

    fn load_css() {
        let display = gdk::Screen::default().expect("Could not get default display");

        let provider = gtk::CssProvider::new();
        provider
            .load_from_data(CSS_DATA)
            .expect("Failed to load CSS");

        gtk::StyleContext::add_provider_for_screen(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn set_dark_theme() {
        if let Some(settings) = Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(true);
        }
    }

    fn create_window(app: &Application) -> ApplicationWindow {
        ApplicationWindow::builder()
            .application(app)
            .title("Power Menu")
            .default_width(WINDOW_WIDTH)
            .resizable(false)
            .build()
    }

    fn setup_window_properties(&self) {
        // Configure window for floating above tiling WM
        self.window.set_type_hint(gdk::WindowTypeHint::Dialog);
        self.window.set_modal(false);
        self.window.set_keep_above(true);
        self.window.set_decorated(false);
        self.window.set_skip_taskbar_hint(true);
        self.window.set_skip_pager_hint(true);

        // Center window and set focus properties
        self.window.set_position(gtk::WindowPosition::Center);
        self.window.set_accept_focus(true);
        self.window.set_focus_on_map(true);

        // Add event masks
        self.window.add_events(
            gdk::EventMask::KEY_PRESS_MASK
                | gdk::EventMask::FOCUS_CHANGE_MASK
                | gdk::EventMask::BUTTON_PRESS_MASK,
        );
    }

    fn setup_event_handlers(&self) {
        // Keyboard shortcuts
        self.window.connect_key_press_event(|_, event| {
            let command = match event.keyval() {
                gdk::keys::constants::s => Some(commands::SHUTDOWN),
                gdk::keys::constants::r => Some(commands::REBOOT),
                gdk::keys::constants::e => Some(commands::EXIT),
                gdk::keys::constants::l => Some(commands::LOCK),
                gdk::keys::constants::h => Some(commands::SUSPEND),
                gdk::keys::constants::Escape | gdk::keys::constants::q => {
                    Self::fast_exit();
                    return Propagation::Stop;
                }
                _ => None,
            };

            if let Some(cmd) = command {
                Self::execute_command_and_exit(cmd);
            }

            Propagation::Stop
        });

        // Close on any click outside (if we add button press mask)
        self.window.connect_button_press_event(|window, event| {
            let (window_width, window_height) = window.size();
            let (x, y) = event.position();

            // If click is outside window bounds, close immediately
            if x < 0.0 || y < 0.0 || x > window_width as f64 || y > window_height as f64 {
                Self::fast_exit();
            }

            Propagation::Proceed
        });
    }

    fn create_ui(&self) {
        let vbox = GtkBox::new(gtk::Orientation::Vertical, 5);
        vbox.set_margin(10);

        // Use slice of tuples for cleaner button definition
        const BUTTONS: &[(&str, &str)] = &[
            ("(s) Shutdown", commands::SHUTDOWN),
            ("(r) Reboot", commands::REBOOT),
            ("(e) Exit", commands::EXIT),
            ("(l) Lock", commands::LOCK),
            ("(h) Suspend", commands::SUSPEND),
        ];

        for &(label, cmd) in BUTTONS {
            let button = self.create_button(label, cmd);
            vbox.pack_start(&button, false, false, 2);
        }

        self.window.add(&vbox);
    }

    fn create_button(&self, label: &str, command: &str) -> Button {
        let button = Button::with_label(label);
        button.set_hexpand(true);
        button.set_vexpand(false);
        button.set_widget_name("tui-button");

        let cmd = command.to_string();
        button.connect_clicked(move |_| {
            Self::execute_command_and_exit(&cmd);
        });

        button
    }

    // Fast exit without GTK cleanup
    fn fast_exit() {
        exit(0);
    }

    // Execute command and exit immediately
    fn execute_command_and_exit(cmd: &str) {
        // Start command in background
        let _ = Command::new("sh").arg("-c").arg(cmd).spawn();

        // Exit immediately without waiting
        Self::fast_exit();
    }

    fn show(&self) {
        self.window.show_all();
        self.window.present();
        self.window.grab_focus();
    }
}

fn main() {
    let application = Application::builder().application_id(APP_ID).build();

    application.connect_activate(|app| {
        PowerMenuApp::new(app);
    });

    application.run();
}
