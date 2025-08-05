use gtk::gdk;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Settings};
use std::process::Command;

fn load_css() {
    let display = gdk::Screen::default().expect("Could not get default display.");
    let provider = gtk::CssProvider::new();
    let priority = gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;

    provider.load_from_data(include_bytes!("../styles/kanagawa.css"));
    gtk::StyleContext::add_provider_for_screen(&display, &provider, priority);
}

fn set_dark_theme() {
    if let Some(settings) = Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }
}

fn execute_command(cmd: &str) {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .expect("Failed to execute command")
        .wait();
    gtk::main_quit();
}

fn main() {
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(|app| {
        load_css();
        set_dark_theme();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Power Menu")
            .default_width(300)
            .default_height(250)
            .resizable(false)
            .build();

        // Configurar la ventana para que flote por encima del tiling WM
        window.set_type_hint(gdk::WindowTypeHint::Dialog);
        window.set_modal(false);
        window.set_keep_above(true);
        window.set_decorated(false);
        window.set_skip_taskbar_hint(true);
        window.set_skip_pager_hint(true);

        // Centrar la ventana en la pantalla
        window.set_position(gtk::WindowPosition::Center);

        // Hacer que la ventana tome el foco
        window.set_urgency_hint(false);
        window.set_accept_focus(true);
        window.set_focus_on_map(true);

        // Configurar atajos de teclado
        window.add_events(gdk::EventMask::KEY_PRESS_MASK | gdk::EventMask::FOCUS_CHANGE_MASK);

        window.connect_key_press_event(move |_, event| {
            match event.keyval() {
                // Shutdown
                gdk::keys::constants::s => execute_command("systemctl poweroff"),
                // Reboot
                gdk::keys::constants::r => execute_command("systemctl reboot"),
                // Exit
                gdk::keys::constants::e => execute_command("swaymsg exit"),
                // Lock
                gdk::keys::constants::l => execute_command(
                    "killall pmenu && swaylock \
	--screenshots \
	--clock \
	--indicator \
	--indicator-radius 100 \
	--indicator-thickness 7 \
	--effect-blur 7x5 \
	", // Killall es para que cuando haga swaylock se quite la ventana antes, debe de haber una mejor forma de hacer esto
                ),

                // Suspend
                gdk::keys::constants::h => execute_command("systemctl suspend"),
                gdk::keys::constants::Escape | gdk::keys::constants::q => gtk::main_quit(),
                _ => (),
            }
            Propagation::Stop
        });

        // Cerrar cuando se cambie de foco
        window.connect_focus_out_event(|window, _| {
            window.close();
            Propagation::Stop
        });

        let vbox = GtkBox::new(gtk::Orientation::Vertical, 5);
        vbox.set_margin_start(5);
        vbox.set_margin_end(5);
        vbox.set_margin_top(5);
        vbox.set_margin_bottom(5);
        window.add(&vbox);

        let buttons = [
            ("(s) Shutdown", "systemctl poweroff"),
            ("(r) Reboot", "systemctl reboot"),
            ("(e) Exit", "swaymsg exit"),
            ("(l) Lock", "swaylock"),
            ("(h) Suspend", "systemctl suspend"),
        ];

        for (label, cmd) in buttons.iter() {
            let button = Button::with_label(label);
            button.set_hexpand(true);
            button.set_vexpand(false);
            button.set_widget_name("tui-button");
            // button.set_size_request(0, 0);

            let command = cmd.to_string();
            button.connect_clicked(move |_| {
                execute_command(&command);
            });
            vbox.pack_start(&button, false, false, 2);
        }

        window.show_all();

        // Asegurar que la ventana esté en primer plano
        window.present();
        window.grab_focus();
    });

    application.run();
}
