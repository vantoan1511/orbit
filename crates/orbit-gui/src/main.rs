use orbit_gui::app::OrbitApp;

fn main() -> iced::Result {
    let hidden = orbit_gui::tray::is_hidden_launch(std::env::args());
    let _instance = single_instance::SingleInstance::new("orbit-desktop-instance");

    iced::application("Orbit", OrbitApp::update, OrbitApp::view)
        .subscription(OrbitApp::subscription)
        .font(orbit_gui::icons::LUCIDE_FONT_BYTES)
        .window(iced::window::Settings {
            size: iced::Size::new(1280.0, 800.0),
            visible: !hidden,
            decorations: false,
            transparent: true,
            ..Default::default()
        })
        .run()
}
