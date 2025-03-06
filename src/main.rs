use std::any::Any;
use std::f32;

use bevy::color::palettes::tailwind;
use bevy::math::ops::atan2;
use bevy::prelude::*;
use bevy::utils::tracing::instrument::WithSubscriber;
use bevy::window::WindowResolution;

fn main() {
    let primary_window = Some(Window {
        title: "Counter DOOM".to_string(),
        resolution: WindowResolution::new(512.0, 512.0),
        resizable: false,
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
        .add_systems(Update, (move_bird, check_collisions))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bird {
    pub velocity: f32,
}

#[derive(Resource)]
struct PipeMan {
    pub pipe_image: Handle<Image>,
    pub window_size: Vec2,
}

impl PipeMan {
    const PIXEL_X: f32 = 18.0;
    const PIXEL_Y: f32 = 144.0;
    const WORLD_Y: f32 = PipeMan::PIXEL_Y * PIXEL_RATIO;
}

const SCROLL_SPEED: f32 = 10.0;

impl Bird {
    const GRAVITY: f32 = -0.0;
    const FLAP_DV: f32 = 1.0;
    const SIZE_PIXEL_Y: f32 = 8.0;
    const SIZE_Y: f32 = Bird::SIZE_PIXEL_Y * PIXEL_RATIO;
}

const PIXEL_RATIO: f32 = 4.0;

#[derive(Component)]
struct Obsticle;

fn spawn_obsticle(pos: Vec2, gap: f32, cmds: &mut Commands, image: Handle<Image>) {
    let y = -PipeMan::PIXEL_Y / 2.0 * PIXEL_RATIO + pos.y - gap / 2.0;

    let translation = Vec3 {
        x: 0.0 + pos.x,
        y,
        z: 0.0,
    };
    let transform = Transform::from_translation(translation).with_scale(Vec3::splat(PIXEL_RATIO));

    let mut transform2 = transform.clone();
    transform2.rotate_around(Vec3::ZERO, Quat::from_rotation_z(std::f32::consts::PI));
    transform2.scale.x *= -1.0;

    let sprite = dbg!(Sprite {
        image,
        ..Default::default()
    });
    let bundle1 = (Obsticle, sprite.clone(), transform);
    let bundle2 = (Obsticle, sprite, transform2);
    cmds.spawn_batch([bundle1, bundle2]);
}

fn check_collisions(
    playerq: Query<&Transform, With<Player>>,
    obsticles: Query<(Entity, &Transform), With<Obsticle>>,
) {
    let player = playerq.single();
    for (ent, obs) in obsticles.iter() {
        let vec3 = Vec3 {
            y: PipeMan::WORLD_Y / 2.0,
            ..default()
        };
        // let vec3 = Vec3::ZERO;
        let diff = obs.rotation * (player.translation - obs.translation) - vec3;
        dbg!(ent, &diff);
    }
}

fn setup_level(winq: Query<&Window>, mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.insert_resource(ClearColor(Color::from(tailwind::TEAL_400)));
    cmds.spawn(Camera2d::default());

    let pipe_image: Handle<Image> = asset_server.load("pipe.png");
    let resource = PipeMan {
        pipe_image: pipe_image.clone(),
        window_size: winq.single().resolution.size(),
    };
    cmds.insert_resource(resource);

    let gap = Bird::SIZE_Y * 2.0;
    spawn_obsticle(Vec2::ZERO, gap, &mut cmds, pipe_image);

    let transform = Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO));
    // transform.translation.y = Bird::BIRD_PX_Y / 2;

    let player_bundle = (
        Player,
        Bird { velocity: 0.0 },
        Sprite {
            image: asset_server.load("bird.png"),
            ..Default::default()
        },
        transform,
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
