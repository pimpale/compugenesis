use gio::prelude::*;
use gio::ApplicationFlags;
use gtk::prelude::*;

use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy)]
pub struct SettingsPacket {
    pub sunlight: f64,
    pub gravity: f64,
    pub moisture: f64,
    pub nitrogen: f64,
    pub potassium: f64,
    pub phosphorus: f64,
}

pub fn gtk_setup(settings_packet: Arc<RwLock<SettingsPacket>>) -> () {
    std::thread::spawn(move || {
        let application = gtk::Application::new(
            "com.github.gtk-rs.examples.basic",
            ApplicationFlags::empty(),
        )
        .expect("Initialization failed...");
        application.connect_activate(move |app| {
            let window = gtk::ApplicationWindow::new(app);

            window.set_title("GUI");
            window.set_border_width(10);
            window.set_position(gtk::WindowPosition::Center);
            window.set_default_size(350, 350);

            let sunlight_scale =
                gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
            let gravity_scale =
                gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 20.0, 0.1);
            let moisture_scale =
                gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);

            sunlight_scale.set_size_request(200, 10);
            gravity_scale.set_size_request(200, 10);
            moisture_scale.set_size_request(200, 10);

            let sunlight_cloned_settings_packet = settings_packet.clone();
            sunlight_scale.connect_value_changed(move |sc| {
                let mut w = sunlight_cloned_settings_packet.write().unwrap();
                w.sunlight = sc.get_value();
            });

            let gravity_cloned_settings_packet = settings_packet.clone();
            gravity_scale.connect_value_changed(move |sc| {
                let mut w = gravity_cloned_settings_packet.write().unwrap();
                w.gravity = sc.get_value();
            });

            let moisture_cloned_settings_packet = settings_packet.clone();
            moisture_scale.connect_value_changed(move |sc| {
                let mut w = moisture_cloned_settings_packet.write().unwrap();
                w.moisture = sc.get_value();
            });

            let sunlight_label = gtk::Label::new("Sunlight");
            let gravity_label = gtk::Label::new("Gravity");
            let moisture_label = gtk::Label::new("Moisture");

            let sunlight = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            let gravity = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            let moisture = gtk::Box::new(gtk::Orientation::Horizontal, 1);

            sunlight.add(&sunlight_label);
            sunlight.add(&sunlight_scale);

            gravity.add(&gravity_label);
            gravity.add(&gravity_scale);

            moisture.add(&moisture_label);
            moisture.add(&moisture_scale);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);

            vbox.add(&sunlight);
            vbox.add(&gravity);
            vbox.add(&moisture);
            window.add(&vbox);
            window.show_all();
        });

        application.run(&[] as &[&str]);
    });
}
