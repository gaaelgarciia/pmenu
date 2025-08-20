use gtk::gdk;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Settings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::{Command, exit};

const APP_ID: &str = "com.example.FirstGtkApp";
const WINDOW_WIDTH: i32 = 300;
const CSS_DATA: &[u8] = include_bytes!("../styles/kanagawa.css");
const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize, Serialize, Clone)]
struct PowerMenuConfig {
    commands: CommandsConfig,
}

#[derive(Deserialize, Serialize, Clone)]
struct CommandsConfig {
    shutdown: CommandEntry,
    reboot: CommandEntry,
    exit: CommandEntry,
    suspend: CommandEntry,
    lock: CommandEntry,
}

#[derive(Deserialize, Serialize, Clone)]
struct CommandEntry {
    label: String,
    command: String,
    #[serde(default)]
    key: Option<char>,
}

impl Default for PowerMenuConfig {
    fn default() -> Self {
        Self {
            commands: CommandsConfig {
                shutdown: CommandEntry {
                    label: "(s) Shutdown".to_string(),
                    command: "systemctl poweroff".to_string(),
                    key: Some('s'),
                },
                reboot: CommandEntry {
                    label: "(r) Reboot".to_string(),
                    command: "systemctl reboot".to_string(),
                    key: Some('r'),
                },
                exit: CommandEntry {
                    label: "(e) Exit".to_string(),
                    command: "swaymsg exit".to_string(),
                    key: Some('e'),
                },
                lock: CommandEntry {
                    label: "(l) Lock".to_string(),
                    command: "swaylock".to_string(),
                    key: Some('l'),
                },
                suspend: CommandEntry {
                    label: "(h) Suspend".to_string(),
                    command: "systemctl suspend".to_string(),
                    key: Some('h'),
                },
            },
        }
    }
}

#[derive(Clone)]
struct PowerMenuApp {
    window: ApplicationWindow,
    config: PowerMenuConfig,
}

impl PowerMenuApp {
    fn new(app: &Application) -> Self {
        let config = Self::load_config();
        Self::load_css();
        Self::set_dark_theme();

        let window = Self::create_window(app);
        let app_instance = Self {
            window: window.clone(),
            config: config.clone(),
        };

        app_instance.setup_window_properties();
        app_instance.setup_event_handlers();
        app_instance.create_ui();

        app_instance.show();
        app_instance
    }

    fn load_config() -> PowerMenuConfig {
        match fs::read_to_string(CONFIG_FILE) {
            Ok(contents) => match toml::from_str::<PowerMenuConfig>(&contents) {
                Ok(config) => {
                    println!("Loaded configuration from {}", CONFIG_FILE);
                    config
                }
                Err(e) => {
                    eprintln!(
                        "Error parsing {}: {}. Using default configuration.",
                        CONFIG_FILE, e
                    );
                    Self::create_default_config();
                    PowerMenuConfig::default()
                }
            },
            Err(_) => {
                println!(
                    "Configuration file {} not found. Creating default configuration.",
                    CONFIG_FILE
                );
                Self::create_default_config();
                PowerMenuConfig::default()
            }
        }
    }

    fn create_default_config() {
        let default_config = PowerMenuConfig::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .expect("Failed to serialize default configuration");

        if let Err(e) = fs::write(CONFIG_FILE, toml_string) {
            eprintln!("Failed to write default configuration file: {}", e);
        } else {
            println!("Created default configuration file: {}", CONFIG_FILE);
        }
    }

    fn load_css() {
        let screen = gdk::Screen::default().expect("Could not get default screen");
        let provider = gtk::CssProvider::new();

        match provider.load_from_data(CSS_DATA) {
            Ok(_) => {
                gtk::StyleContext::add_provider_for_screen(
                    &screen,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
            Err(e) => {
                eprintln!("Failed to load CSS: {}", e);
            }
        }
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
        let config = self.config.clone();

        // Keyboard shortcuts
        self.window.connect_key_press_event(move |_, event| {
            let keyval = event.keyval();

            // Handle Escape and Q keys using raw key values
            if keyval == gdk::keys::constants::Escape || keyval == gdk::keys::constants::q {
                Self::fast_exit();
                return Propagation::Stop;
            }

            // Convert keyval to char and check against configured keys
            if let Some(key_char) = keyval.to_unicode() {
                let key_char_lower = key_char.to_lowercase().next().unwrap_or(key_char);

                let command = [
                    &config.commands.shutdown,
                    &config.commands.reboot,
                    &config.commands.exit,
                    &config.commands.lock,
                    &config.commands.suspend,
                ]
                .iter()
                .find(|cmd| cmd.key == Some(key_char_lower))
                .map(|cmd| cmd.command.clone());

                if let Some(cmd) = command {
                    Self::execute_command_and_exit(&cmd);
                }
            }

            Self::fast_exit();
            Propagation::Stop
        });

        // Close on any click outside
        self.window.connect_button_press_event(|window, event| {
            let (window_width, window_height) = window.size();
            let (x, y) = event.position();

            if x < 0.0 || y < 0.0 || x > window_width as f64 || y > window_height as f64 {
                Self::fast_exit();
            }

            Propagation::Proceed
        });
    }

    fn create_ui(&self) {
        let vbox = GtkBox::new(gtk::Orientation::Vertical, 5);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);

        // Create buttons from configuration
        let commands = [
            &self.config.commands.shutdown,
            &self.config.commands.reboot,
            &self.config.commands.exit,
            &self.config.commands.lock,
            &self.config.commands.suspend,
        ];

        for cmd in commands.iter() {
            let button = self.create_button(&cmd.label, &cmd.command);
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

    // Fast exit without GTK cleanup - like wofi
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
