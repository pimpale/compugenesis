use std::sync::Arc;
use std::sync::RwLock;

use gio::prelude::*;
use gio::ApplicationFlags;

use gtk::prelude::*;
use gtk::MenuItemExt;

#[derive(Debug, Clone, Copy)]
pub struct SettingsPacket {
    pub paused: bool,
    pub request_stop: bool,
    pub requested_fps: Option<u32>,
    pub simulation_duration: Option<u32>, //In cycles
}

pub fn gtk_run(settings_packet: Arc<RwLock<SettingsPacket>>) -> () {
    let application = gtk::Application::new(
        "com.github.gtk-rs.examples.basic",
        ApplicationFlags::empty(),
    )
    .expect("Initialization failed...");
    application.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);

        window.set_title("CompuGenesis");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 350);

        simulation_screen(window.clone(), settings_packet.clone());

        window.set_default_size(350, 350);
    });

    application.run(&[] as &[&str]);
}

fn welcome_screen(
    window: gtk::ApplicationWindow,
    settings_packet: Arc<RwLock<SettingsPacket>>,
) -> () {

}

fn simulation_screen(
    window: gtk::ApplicationWindow,
    settings_packet: Arc<RwLock<SettingsPacket>>,
) -> () {
    let menu_bar = gtk::MenuBar::new();
    let file_menu_item = gtk::MenuItem::new_with_label("File");
    let file_menu = gtk::Menu::new();

    let view_menu_item = gtk::MenuItem::new_with_label("View");
    let settings_menu_item = gtk::MenuItem::new_with_label("Settings");
    let help_menu_item = gtk::MenuItem::new_with_label("Help");

    let import_menu_item = gtk::MenuItem::new_with_label("Import simulation");
    let export_menu_item = gtk::MenuItem::new_with_label("Export simulation");
    let quit_menu_item = gtk::MenuItem::new_with_label("Quit");

    file_menu_item.set_submenu(&file_menu);
    file_menu.append(&import_menu_item);
    file_menu.append(&export_menu_item);
    file_menu.append(&quit_menu_item);

    menu_bar.append(&file_menu_item);
    menu_bar.append(&view_menu_item);
    menu_bar.append(&settings_menu_item);
    menu_bar.append(&help_menu_item);

    let fps_scale = gtk::Scale::new_with_range(gtk::Orientation::Horizontal, -1.0, 55.0, 2.0);

    fps_scale.set_size_request(200, 10);

    let fps_cloned_settings_packet = settings_packet.clone();
    fps_scale.connect_value_changed(move |sc| {
        let mut w = fps_cloned_settings_packet.write().unwrap();
        w.requested_fps = if sc.get_value() < 0.0 {
            None
        } else {
            Some(sc.get_value() as u32)
        };
    });

    let duration_cloned_settings_packet = settings_packet.clone();

    let fps_label = gtk::Label::new("Compute cycles per second");

    let fps = gtk::Box::new(gtk::Orientation::Vertical, 1);

    fps.add(&fps_label);
    fps.add(&fps_scale);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
    vbox.add(&menu_bar);
    vbox.add(&fps);

    window.add(&vbox);
    window.show_all();
}
