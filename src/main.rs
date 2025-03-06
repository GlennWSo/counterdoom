use bevy::color::palettes::tailwind;
use bevy::math::ops::atan2;
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

    App::new()
        .add_plugins(default_plugins)
        .add_systems(Startup, setup_level)
        .add_systems(Update, move_bird)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bird {
    pub velocity: f32,
}

const SCROLL_SPEED: f32 = 10.0;

impl Bird {
    const GRAVITY: f32 = -3.0;
    const FLAP_DV: f32 = 1.0;
}

const PIXEL_RATIO: f32 = 4.0;

fn setup_level(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.insert_resource(ClearColor(Color::from(tailwind::TEAL_400)));
    cmds.spawn(Camera2d::default());

    let player_bundle = (
        Player,
        Bird { velocity: 0.0 },
        Sprite {
            image: asset_server.load("bird.png"),
            ..Default::default()
        },
        Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO)),
    );
    cmds.spawn(player_bundle);
}

fn move_bird(
    mut birdq: Query<(&mut Bird, &mut Transform), With<Player>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut bird, mut transform)) = birdq.get_single_mut() else {
        return;
    };
    // dbg!(bird.velocity);

    let dt = time.delta_secs();

    if keys.just_pressed(KeyCode::Space) {
        bird.velocity = Bird::FLAP_DV;
        dbg!("flap");
        dbg!(bird.velocity);
    }

    bird.velocity += dt * Bird::GRAVITY;
    transform.translation.y += bird.velocity;
    let angle = atan2(bird.velocity * 2.0, SCROLL_SPEED);
    transform.rotation = Quat::from_axis_angle(Vec3::Z, angle)
    // transform.translation.x += 0 * dt;
}
