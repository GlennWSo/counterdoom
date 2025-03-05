use bevy::prelude::*;

fn main() {
    let primary_window = Some(Window {
        title: "Counter DOOM".to_string(),
        ..Default::default()
    });

    let default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window,
            ..Default::default()
        })
        .set(ImagePlugin::default_nearest());

    App::new().add_plugins(default_plugins).run();
}
