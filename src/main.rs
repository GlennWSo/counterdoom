use std::f32::consts::PI;

use bevy::math::ops::atan2;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::{color::palettes::tailwind, utils::tracing::Instrument};
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
            (
                move_player,
                player_use_kit,
                player_change_kit, // derp
            )
                .run_if(in_state(GameState::Playing)),
        )
        // .add_systems(Update, toogle_game_state)
        .add_systems(Update, setup_level.run_if(in_state(GameState::Spawn)))
        .run();
}
// const SCROLL_SPEED: f32 = 200.0;
// const WORLD_SIZE: f32 = 8000.0;
const PIXEL_RATIO: f32 = 4.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Hero;

#[derive(Component)]
struct Walk {
    speed: f32,
}

impl Walk {
    fn new(speed: f32) -> Self {
        Self { speed }
    }
}

#[derive(Component, Default)]
struct ToolKit {
    slots: [Option<Entity>; 9],
    active: u8,
}

impl ToolKit {
    fn new(stuff: Vec<Entity>) -> Self {
        let slots = [
            stuff.get(0).map(|e| *e),
            stuff.get(1).map(|e| *e),
            stuff.get(2).map(|e| *e),
            stuff.get(3).map(|e| *e),
            stuff.get(4).map(|e| *e),
            stuff.get(5).map(|e| *e),
            stuff.get(6).map(|e| *e),
            stuff.get(7).map(|e| *e),
            stuff.get(8).map(|e| *e),
        ];
        Self { slots, active: 0 }
    }

    fn get_active(&self) -> Option<Entity> {
        let res = self.slots.get(self.active as usize);
        match res {
            Some(res) => *res,
            None => None,
        }
    }
}

#[derive(Component)]
struct FireBlaster;

fn player_change_kit(
    mut playerq: Query<&mut ToolKit, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut kit) = playerq.get_single_mut() else {
        return;
    };

    for key in keys.get_pressed() {
        match key {
            KeyCode::Digit1 => kit.active = 0,
            KeyCode::Digit2 => kit.active = 1,
            KeyCode::Digit3 => kit.active = 2,
            KeyCode::Digit4 => kit.active = 3,
            KeyCode::Digit5 => kit.active = 4,
            KeyCode::Digit6 => kit.active = 5,
            KeyCode::Digit7 => kit.active = 6,
            KeyCode::Digit8 => kit.active = 7,
            KeyCode::Digit9 => kit.active = 8,
            _ => (),
        }
    }
}

fn player_use_kit(playerq: Query<&ToolKit, With<Player>>, keys: Res<ButtonInput<KeyCode>>) {
    let Ok(kit) = playerq.get_single() else {
        return;
    };

    let Some(thing) = kit.get_active() else {
        return;
    };

    if keys.pressed(KeyCode::Space) {
        // dbg!("try using", thing);
        return;
    }
}

fn move_player(
    mut playerq: Query<(&mut Transform, &Walk), With<Player>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut transform, Walk { speed })) = playerq.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    for pressed in keys.get_pressed() {
        match pressed {
            KeyCode::ArrowUp | KeyCode::KeyW => direction += Vec2 { x: 0.0, y: 1.0 },
            KeyCode::ArrowDown | KeyCode::KeyS => direction += Vec2 { x: 0.0, y: -1.0 },
            KeyCode::ArrowLeft | KeyCode::KeyA => direction += Vec2 { x: -1.0, y: 0.0 },
            KeyCode::ArrowRight | KeyCode::KeyD => direction += Vec2 { x: 1.0, y: 0.0 },
            _ => (),
        }
    }
    if direction == Vec2::ZERO {
        return;
    }

    let dt = time.delta_secs();
    transform.translation += (direction.normalize() * speed * dt).extend(0.0);
}

fn setup_level(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // spawn the player ctrled hero character
    let hero_sprite = Sprite {
        image: asset_server.load("bird.png"),
        ..Default::default()
    };

    let fire_blaster = cmds.spawn(FireBlaster).id();

    let toolkit = ToolKit::new(vec![fire_blaster]);

    let hero = (
        hero_sprite,
        toolkit,
        Hero,
        Walk::new(100.0),
        Player,
        Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO)),
    );
    cmds.spawn(hero).add_child(fire_blaster);

    // spawn a pipe

    next_state.set(GameState::Playing);
}

fn setup_game(
    mut cmds: Commands,
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
    // let pipe_image: Handle<Image> = asset_server.load("pipe.png");

    next_state.set(GameState::Spawn)
}
