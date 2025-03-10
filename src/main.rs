use std::f32::consts::PI;
use std::fmt;
use std::ops::{DerefMut, Sub, SubAssign};

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
        .add_event::<TryTool>()
        .add_systems(
            Update,
            (
                move_player,
                player_use_kit,
                player_change_kit, // derp
                // debug_tool_use,
                use_tool::<FireBlastSpell>,
                cooldown_system::<FireBlastSpell>,
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

#[derive(Component, Default, Debug)]
struct Hero;

#[derive(Component, Default, Debug, PartialEq, PartialOrd)]
struct Vitals {
    mana: u8,
    life: f32,
    stamina: f32,
}

impl Sub for Vitals {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vitals {
            mana: self.mana - rhs.mana,
            life: self.life - rhs.life,
            stamina: self.stamina - rhs.stamina,
        }
    }
}

impl SubAssign<&Self> for Vitals {
    fn sub_assign(&mut self, rhs: &Vitals) {
        self.mana -= rhs.mana;
        self.life -= rhs.life;
        self.stamina -= rhs.stamina;
    }
}

/// Things like fireball skill
/// or sword attack etc
/// Doge roll?
trait CombatSkill {
    fn preform(&mut self, cmds: &mut Commands, origin: &Transform, asset_server: &Res<AssetServer>);
    fn cost(&self) -> Vitals {
        Vitals::default()
    }
    fn ready(&self) -> bool {
        true
    }
}

trait Cooldown {
    fn cooldown(&mut self, dt: f32);
    fn ready(&self) -> bool;
}

// #[derive(Component, Default, Debug, Deref, DerefMut, PartialEq, PartialOrd)]
// struct Life(f32);
// #[derive(Component, Default, Debug, Deref, DerefMut, PartialEq, PartialOrd)]
// struct Stamina(f32);
// #[derive(Component, Default, Debug, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
// struct Mana(u32);

#[derive(Component)]
struct Walk {
    speed: f32,
}

#[derive(Component, Default, Debug)]
struct Cost {
    vitals: Vitals,
}

impl Vitals {
    fn with_mana(mut self, mana: u8) -> Self {
        self.mana = mana;
        self
    }
    fn with_life(mut self, life: f32) -> Self {
        self.life = life;
        self
    }
    fn with_stamina(mut self, stamina: f32) -> Self {
        self.stamina = stamina;
        self
    }
}
impl Cost {
    fn with_mana(mut self, mana: u8) -> Self {
        self.vitals.mana = mana;
        self
    }
    fn with_life(mut self, life: f32) -> Self {
        self.vitals.life = life;
        self
    }
    fn with_stamina(mut self, stamina: f32) -> Self {
        self.vitals.stamina = stamina;
        self
    }
}

#[derive(Event)]
struct TryTool {
    tool: Entity,
    user: Entity,
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

#[derive(Component, Default, Debug)]
struct FireBlastSpell {
    cooldown: f32,
}

#[derive(Component, Default, Debug)]
struct FireBolt {
    velocity: f32,
    /// the destrucive capability
    power: f32,
}

// #[derive(Component, Debug, Deref, DerefMut)]
// struct Cooldown(f32);

impl FireBlastSpell {
    const COOLDOWN: f32 = 1.0;
}

impl CombatSkill for FireBlastSpell {
    fn preform(
        &mut self,
        cmds: &mut Commands,
        origin: &Transform,
        asset_server: &Res<AssetServer>,
    ) {
        println!("Fire blast at {:#?}", origin.translation);
        self.cooldown = FireBlastSpell::COOLDOWN;
        let image = asset_server.load("bird.png");

        let sprite = Sprite {
            image,
            color: Color::Srgba(Srgba {
                red: 0.5,
                green: 0.0,
                blue: 0.0,
                alpha: 0.5,
            }),
            ..Default::default()
        };

        cmds.spawn((
            sprite,
            origin.clone(),
            FireBolt {
                velocity: 1000.0,
                power: 30.0,
            },
        ));
    }

    fn ready(&self) -> bool {
        self.cooldown <= 0.0
    }

    fn cost(&self) -> Vitals {
        Vitals::default().with_stamina(10.0)
    }
}

impl Cooldown for FireBlastSpell {
    fn cooldown(&mut self, dt: f32) {
        self.cooldown -= dt;
    }

    fn ready(&self) -> bool {
        CombatSkill::ready(self)
    }
}

fn cooldown_system<T: Cooldown + Component>(mut q: Query<&mut T>, time: Res<Time>) {
    let dt = time.delta_secs();
    for mut cooler in q.iter_mut() {
        if cooler.ready() {
            continue;
        }
        cooler.cooldown(dt);
    }
}

fn player_change_kit(
    mut playerq: Query<&mut ToolKit, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut kit) = playerq.get_single_mut() else {
        return;
    };

    for key in keys.get_just_pressed() {
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

fn use_tool<T: CombatSkill + Component + fmt::Debug>(
    mut cmds: Commands,
    mut tool_uses: EventReader<TryTool>,
    mut q_ability: Query<&mut T>,
    mut user_q: Query<(&mut Vitals, &Transform)>,
    asset_server: Res<AssetServer>,
) {
    for TryTool { user, tool } in tool_uses.read() {
        let Ok(mut ability) = q_ability.get_mut(*tool) else {
            continue;
        };

        let Ok((mut vitals, transform)) = user_q.get_mut(*user) else {
            continue;
        };
        let cost = ability.cost();
        if !(vitals.as_ref() >= &cost) {
            println!("not enough vitals");
            continue;
        }
        if !ability.ready() {
            println!("ability not ready");
            continue;
        }

        *vitals -= &cost;
        dbg!(vitals);
        ability.preform(&mut cmds, transform, &asset_server);
    }
}

fn player_use_kit(
    playerq: Query<(Entity, &ToolKit), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut try_tool: EventWriter<TryTool>,
) {
    let Ok((user, kit)) = playerq.get_single() else {
        return;
    };
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let Some(thing) = kit.get_active() else {
        println!("nothing is equiped"); // dbg
        return;
    };
    try_tool.send(TryTool { tool: thing, user });
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

    let fire_blaster = cmds
        .spawn((
            FireBlastSpell::default(),
            Cost::default().with_stamina(10.0),
        ))
        .id();

    let toolkit = ToolKit::new(vec![fire_blaster]);

    let hero = (
        hero_sprite,
        toolkit,
        Hero,
        Vitals {
            mana: 3,
            life: 100.0,
            stamina: 100.0,
        },
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
