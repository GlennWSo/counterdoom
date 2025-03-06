use std::f32::consts::PI;

use bevy::color::palettes::tailwind;
use bevy::math::ops::atan2;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use rand::{rng, thread_rng, Rng};

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum GameState {
    #[default]
    Paused,
    Playing,
    GameOver,
    CleanUp,
    Spawn,
}

fn main() {
    let primary_window = Some(Window {
        title: "Counter DOOM".to_string(),
        resolution: WindowResolution::new(800.0, 512.0),
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
        .add_systems(Startup, setup_game)
        .init_state::<GameState>()
        .add_systems(
            Update,
            (move_bird, check_collisions, side_scroll).run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, toogle_game_state)
        .add_systems(Update, despawn_all.run_if(in_state(GameState::CleanUp)))
        .add_systems(Update, setup_level.run_if(in_state(GameState::Spawn)))
        .run();
}
const SCROLL_SPEED: f32 = 200.0;
const WORLD_SIZE: f32 = 8000.0;

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

impl Bird {
    const GRAVITY: f32 = -2.0;
    const FLAP_DV: f32 = 1.5;
    const SIZE_PIXEL_Y: f32 = 8.0;
    const SIZE_Y: f32 = Bird::SIZE_PIXEL_Y * PIXEL_RATIO;
    const SIZE_PIXEL_X: f32 = 12.0;
    const SIZE_X: f32 = Bird::SIZE_PIXEL_X * PIXEL_RATIO;
}

const PIXEL_RATIO: f32 = 4.0;

#[derive(Component)]
struct Obsticle;

enum ObsticlePair {
    Top,
    Bot,
    Both,
}

fn spawn_obsticle(
    pos: Vec2,
    gap: f32,
    pair: ObsticlePair,
    cmds: &mut Commands,
    image: Handle<Image>,
) {
    let Vec2 { x, y } = pos;

    let y0 = -PipeMan::PIXEL_Y / 2.0 * PIXEL_RATIO - gap / 2.0;

    let translation = Vec3 { x, y: y0, z: 0.0 };
    let mut transform =
        Transform::from_translation(translation).with_scale(Vec3::splat(PIXEL_RATIO));

    let mut transform2 = transform.clone();
    // transform2.rotate_around(Vec3::ZERO, Quat::from_rotation_z(std::f32::consts::PI));
    transform2.rotate_z(PI);
    transform2.translation.y *= -1.0;
    transform2.scale.x *= -1.0;

    transform.translation.y += y;
    transform2.translation.y += y;

    let sprite = dbg!(Sprite {
        image,
        ..Default::default()
    });
    let bundle1 = (Obsticle, sprite.clone(), transform);
    let bundle2 = (Obsticle, sprite, transform2);

    match pair {
        ObsticlePair::Top => {
            cmds.spawn(bundle2);
        }
        ObsticlePair::Bot => {
            cmds.spawn(bundle1);
        }
        ObsticlePair::Both => {
            cmds.spawn_batch([bundle1, bundle2]);
        }
    }
}

fn side_scroll(
    mut obsticles: Query<&mut Transform, With<Obsticle>>,
    time: Res<Time>,
    pipe_man: Res<PipeMan>,
) {
    let dt = time.delta_secs();
    for mut obs in obsticles.iter_mut() {
        obs.translation.x -= SCROLL_SPEED * dt;
        if obs.translation.x < -300.0 {
            obs.translation.x = WORLD_SIZE;
        }
    }
}

fn toogle_game_state(
    // mut cmds: Commands,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    // q: Query<Entity, (With<Player>, With<Obsticle>)>,
    // winq: Query<&Window>,
    // asset_server: Res<AssetServer>,
    // pipe_man: Res<PipeMan>,
) {
    let state = state.get();
    if keys.just_pressed(KeyCode::Space) && state == &GameState::Paused {
        next_state.set(GameState::Playing);
        return;
    }

    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    dbg!("Escp");
    dbg!(state);

    match state {
        GameState::Paused => (),
        GameState::Playing => next_state.set(GameState::Paused),
        GameState::GameOver => {
            next_state.set(GameState::CleanUp);
        }
        _ => {}
    }
}

fn despawn_all(
    mut cmds: Commands,
    playerq: Query<Entity, With<Player>>,
    obsticles: Query<Entity, With<Obsticle>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for ent in playerq.iter().chain(obsticles.iter()) {
        dbg!(ent);
        cmds.entity(ent).despawn_recursive();
    }
    next_state.set(GameState::Spawn)
}

fn check_collisions(
    playerq: Query<&Transform, With<Player>>,
    obsticles: Query<(Entity, &Transform), With<Obsticle>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player = playerq.single();
    for (ent, obs) in obsticles.iter() {
        let vec3 = Vec3 {
            y: PipeMan::WORLD_Y / 2.0,
            ..default()
        };
        // let vec3 = Vec3::ZERO;
        let diff = obs.rotation * (player.translation - obs.translation) - vec3;

        let in_x = (diff.x.abs() - Bird::SIZE_X) < 0.0;
        let in_y = (diff.y - Bird::SIZE_Y / 2.0) < 0.0;
        dbg!(ent, &diff);
        if in_x && in_y {
            dbg!("collide!");
            match state.get() {
                GameState::Playing => {
                    next_state.set(GameState::GameOver);
                }
                _ => {
                    panic!("derp")
                }
            }
        }
    }
}

fn setup_obsticles(cmds: &mut Commands, image: Handle<Image>) {
    let n = 15;
    let mut x = 500.0;
    let dx = (WORLD_SIZE - x) / n as f32;
    let mut rng = rng();
    for i in 0..n {
        let y = rng.random_range(-100.0..100.);
        spawn_obsticle(
            Vec2::new(x, y),
            Bird::SIZE_Y * 3.5,
            ObsticlePair::Both,
            cmds,
            image.clone(),
        );
        x += dx;
    }
}

fn setup_game(
    mut cmds: Commands,
    winq: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    cmds.insert_resource(ClearColor(Color::from(tailwind::TEAL_400)));
    let transform = Transform::from_translation(Vec3 {
        x: 200.0,
        ..Default::default()
    });
    cmds.spawn(Camera2dBundle {
        transform,
        ..Default::default()
    });
    let pipe_image: Handle<Image> = asset_server.load("pipe.png");
    let resource = PipeMan {
        pipe_image: pipe_image.clone(),
        window_size: winq.single().resolution.size(),
    };
    cmds.insert_resource(resource);

    next_state.set(GameState::Spawn)
}

fn setup_level(
    mut cmds: Commands,
    pipe_man: Res<PipeMan>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let pipe_image = pipe_man.pipe_image.clone();
    setup_obsticles(&mut cmds, pipe_image);

    let transform = Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO));
    // transform.translation.y = Bird::BIRD_PX_Y / 2;

    let player_bundle = (
        Player,
        Bird {
            velocity: Bird::FLAP_DV,
        },
        Sprite {
            image: asset_server.load("bird.png"),
            ..Default::default()
        },
        transform,
    );
    cmds.spawn(player_bundle);
    next_state.set(GameState::Paused);
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
    let angle = atan2(bird.velocity * 10.0, SCROLL_SPEED);
    transform.rotation = Quat::from_axis_angle(Vec3::Z, angle)
    // transform.translation.x += 0 * dt;
}
