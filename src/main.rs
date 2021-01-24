use bevy::{ prelude::*,
    sprite::collide_aabb::{collide},};
//use bevy::diagnostic::{FrameTimeDiagnosticsPlugin,PrintDiagnosticsPlugin};
use rand::prelude::*;

const SCREEN_WIDTH: f32 = 500.0;
const SCREEN_HEIGHT: f32 = 500.0;

const FERRIS_WIDTH: f32 = 48.0;
const FERRIS_HEIGHT: f32 = 32.0;
const WALL_THICKNESS: f32 = 10.0;

const FERRIS_POSITION: f32 = (-(SCREEN_HEIGHT as f32)/2.0)+FERRIS_HEIGHT as f32;
const FERRIS_MAX_POSITION: f32 = (SCREEN_WIDTH as f32/2.0) - WALL_THICKNESS - (FERRIS_WIDTH as f32 / 2.0);

const BUG_WIDTH: u32 = 48;
const BUG_POSITION: f32=  -FERRIS_POSITION-32.0;

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_resource(WindowDescriptor {
            title: "Ferris Dodger".to_string(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            vsync: true,
            resizable: false,
            //mode: WindowMode::Fullscreen { use_size: false },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(DodgerPlugin)
        //.add_plugin(PrintDiagnosticsPlugin::default())                                                                                                                                                                                                                    
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())                                                                                                                                                                                                                                                                                                                                                                                                                                     
        //.add_system(PrintDiagnosticsPlugin::print_diagnostics_system.system())    
        .run();
}

pub struct DodgerPlugin;

impl Plugin for DodgerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_resource(State::default())
            .add_startup_system(setup.system())
            .add_system(ferris_movement_system.system())
            .add_system(bug_movement_system.system())
            .add_system(text_display_system.system())
            .add_system(bug_collision_system.system())
            .add_system(bug_spawn_system.system())
            .add_system(restart_system.system());
    }
}

struct State {
    bug_texture_handle: Handle<ColorMaterial>,
    timer: Timer,
    mode: Mode,
    score: usize,
    bug_sound: Handle<AudioSource>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            bug_texture_handle: Handle::default(),
            timer: Timer::from_seconds(2.0, true),
            mode: Mode::Play,
            score: 0,
            bug_sound: Handle::default(),
        }
    }
}

struct Ferris {
    speed: f32,
}

struct Bug{
    speed: f32,
}

#[derive(Copy, Clone)]
enum Collider {
    Death,
    Scorable,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Mode {
    Play,
    Stop,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum TextMsg {
    Score,
    FinalScore,
}


fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<State>,
) {
    let ferris_texture_handle = asset_server.load("rustacean-orig-noshadow-small.png");
    let font = asset_server.load("FiraSans-Bold.ttf");
    
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        // scoreboard
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                border: Rect::all(Val::Px(5.0)),
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    font:font.clone(),
                    value: "Score:".to_string(),
                    style: TextStyle {
                        color: Color::WHITE,
                        font_size: 28.0,
                        ..TextStyle::default()
                    },
                },
                style: Style {
                    size: Size::new(Val::Percent(100.0),Val::Px(30.0)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(TextMsg::Score)
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(200.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with_children(|parent| {
                // death messages
                parent.spawn(TextBundle {
                text: Text {
                    font,
                    value: String::new(),
                    style: TextStyle {
                        color: Color::WHITE,
                        font_size: 28.0,
                        ..TextStyle::default()
                    },
                },
                style: Style {
                    size: Size::new(Val::Percent(60.0), Val::Px(200.0)),
                    ..Default::default()
                },
                 ..Default::default()
            })
            .with(TextMsg::FinalScore);
        });
        })
         .spawn(SpriteBundle {
            material: materials.add(ferris_texture_handle.into()),
            transform: Transform::from_translation(Vec3::new(0.0, FERRIS_POSITION, 0.0)),
            ..Default::default()
        })
        .with(Ferris { speed: 500.0 })
        .with(Collider::Death)
        ;

    let bug_texture_handle =  materials.add(asset_server.load("bug.png").into());
    state.bug_texture_handle=bug_texture_handle;

    // Add walls
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    
    let bounds = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

    commands
        // left
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(-bounds.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(WALL_THICKNESS, bounds.y + WALL_THICKNESS)),
            ..Default::default()
        })
        // right
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(bounds.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(WALL_THICKNESS, bounds.y + WALL_THICKNESS)),
            ..Default::default()
        })
        // bottom
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -bounds.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x + WALL_THICKNESS, WALL_THICKNESS)),
            ..Default::default()
        })
        .with(Collider::Scorable)
        // top
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, bounds.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x + WALL_THICKNESS, WALL_THICKNESS)),
            ..Default::default()
        })
        ;

        state.bug_sound = asset_server
            .load("sfx_sounds_powerup6.wav");
           
}

fn ferris_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    state: Res<State>,
    mut query: Query<(&Ferris, &mut Transform)>,
) {
    if state.mode != Mode::Play {
        return;
    }
    for (ferris, mut transform) in &mut query.iter_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += 1.0;
        }

        // move the paddle horizontally
        transform.translation.x += time.delta_seconds() * direction * ferris.speed;
        // bound the paddle within the walls
        transform.translation.x = transform.translation.x.min(FERRIS_MAX_POSITION).max(-FERRIS_MAX_POSITION);
    }
}

fn restart_system (commands: &mut Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    bug_query: Query<(Entity, &Bug)>,
    mut query: Query<(&Ferris, &mut Transform)>,) {
        if state.mode != Mode::Stop {
            return;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            state.score=0;
            for (bug_entity, _bug) in &mut bug_query.iter() {
                commands.despawn(bug_entity);
            }
            for (_ferris, mut transform) in &mut query.iter_mut() {
                transform.translation.x=0.0;
            }
            state.timer.set_duration(2.0);
            state.mode=Mode::Play;
        }
}

fn bug_movement_system(time: Res<Time>,
    state: Res<State>,
    mut query: Query<(&Bug, &mut Transform)>) {
        if state.mode != Mode::Play {
            return;
        }

    for (bug, mut transform) in &mut query.iter_mut() {
       transform.translation.y -= time.delta_seconds() * bug.speed;
    }
}

fn bug_spawn_system(commands: &mut Commands,
    time: Res<Time>, mut state: ResMut<State>,){
    if state.mode != Mode::Play {
        return;
    }
    
    state.timer.tick(time.delta_seconds());

    
    if state.timer.finished() {
        let mut rng = thread_rng();
        let x:i32 = rng.gen_range(-5, 5)*BUG_WIDTH as i32 + BUG_WIDTH as i32/2;

        let speed = 40.0 + state.score as f32/1.5;

        let nd = state.timer.duration() * 0.98;
        state.timer.set_duration(nd);

        commands
            .spawn(SpriteBundle {
                material: state.bug_texture_handle.clone(),
                transform: Transform::from_translation(Vec3::new(x as f32, BUG_POSITION, 0.0)),
                ..Default::default()
            })
            .with(Bug {speed})
            ;
    }
}

fn text_display_system(state: Res<State>, mut query: Query<(&TextMsg, &mut Text, &mut Style)>) {
    for (msg,mut text, mut style) in &mut query.iter_mut() {
        match msg {
            TextMsg::Score => {
                text.value = format!("Score: {}", state.score);
            },
            TextMsg::FinalScore => {
                if state.mode == Mode::Stop {
                    text.value = format!("You panic! Final score: {}\nPress <space> to restart!", state.score);
                }
                
                //draw.is_visible = state.mode == Mode::Stop;
                if state.mode == Mode::Stop {
                    style.display=Display::Flex;
                } else {
                    style.display=Display::None;
                }
            },
        }
       
    }
}

fn bug_collision_system(
    commands: &mut Commands,
    mut state: ResMut<State>,
    audio_output: Res<Audio>,
    bug_query: Query<(Entity, &Bug, &Transform, &Sprite)>,
    collider_query: Query<(&Collider, &Transform, &Sprite)>,
    
) {
    for (bug_entity, _bug, ball_transform, sprite) in &mut bug_query.iter() {
        let ball_size = sprite.size;
      
        // check collision with walls
        for (collider, transform, sprite) in &mut collider_query.iter() {
            let collision = collide(
                ball_transform.translation,
                ball_size,
                transform.translation,
                sprite.size,
            );
            if let Some(_collision) = collision {
                // scorable colliders should be despawned and increment the scoreboard on collision
                match *collider {
                    Collider::Scorable => {
                        audio_output.play(state.bug_sound.clone());
                        state.score += 1;
                        commands.despawn(bug_entity);
                    },
                    Collider::Death => {
                        state.mode=Mode::Stop;
                    },
                }

                break;
            }
        }
    }
}