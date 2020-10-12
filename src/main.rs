use bevy::{ prelude::*,
    sprite::collide_aabb::{collide},};
use rand::prelude::*;

const SCREEN_WIDTH: u32 = 500;
const SCREEN_HEIGHT: u32 = 500;

const FERRIS_WIDTH: u32 = 48;
const FERRIS_HEIGHT: u32 = 32;
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
        .add_default_plugins()
        .add_plugin(DodgerPlugin)
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<State>,
) {
    let ferris_texture_handle = asset_server.load("assets/rustacean-orig-noshadow-small.png").unwrap();
    let font = asset_server.load("assets/FiraSans-Bold.ttf").unwrap();
    
    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        // scoreboard
        .spawn(NodeComponents {
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
            parent.spawn(TextComponents {
                text: Text {
                    font,
                    value: "Score:".to_string(),
                    style: TextStyle {
                        color: Color::WHITE,
                        font_size: 28.0,
                    },
                },
                style: Style {
                    size: Size::new(Val::Percent(100.0),Val::Px(30.0)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(TextMsg::Score)
            .spawn(NodeComponents {
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
                parent.spawn(TextComponents {
                text: Text {
                    font,
                    value: String::new(),
                    style: TextStyle {
                        color: Color::WHITE,
                        font_size: 28.0,
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
         .spawn(SpriteComponents {
            material: materials.add(ferris_texture_handle.into()),
            transform: Transform::from_translation(Vec3::new(0.0, FERRIS_POSITION, 0.0)),
            ..Default::default()
        })
        .with(Ferris { speed: 500.0 })
        .with(Collider::Death)
        ;

    let bug_texture_handle =  materials.add(asset_server.load("assets/bug.png").unwrap().into());
    state.bug_texture_handle=bug_texture_handle;

    // Add walls
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    
    let bounds = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

    commands
        // left
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(-bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(WALL_THICKNESS, bounds.y() + WALL_THICKNESS)),
            ..Default::default()
        })
        // right
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(WALL_THICKNESS, bounds.y() + WALL_THICKNESS)),
            ..Default::default()
        })
        // bottom
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(0.0, -bounds.y() / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x() + WALL_THICKNESS, WALL_THICKNESS)),
            ..Default::default()
        })
        .with(Collider::Scorable)
        // top
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(0.0, bounds.y() / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x() + WALL_THICKNESS, WALL_THICKNESS)),
            ..Default::default()
        })
        ;

        state.bug_sound = asset_server
            .load("assets/sfx_sounds_powerup6.wav")
            .unwrap();
           
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
    for (ferris, mut transform) in &mut query.iter() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += 1.0;
        }

        let translation = transform.translation_mut();
        // move the paddle horizontally
        *translation.x_mut() += time.delta_seconds * direction * ferris.speed;
        // bound the paddle within the walls
        *translation.x_mut() = translation.x().min(FERRIS_MAX_POSITION).max(-FERRIS_MAX_POSITION);
    }
}

fn restart_system (mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    mut bug_query: Query<(Entity, &Bug)>,
    mut query: Query<(&Ferris, &mut Transform)>,) {
        if state.mode != Mode::Stop {
            return;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            state.score=0;
            for (bug_entity, _bug) in &mut bug_query.iter() {
                commands.despawn(bug_entity);
            }
            for (_ferris, mut transform) in &mut query.iter() {
                let translation = transform.translation_mut();
                *translation.x_mut()=0.0;
            }
            state.timer.duration=2.0;
            state.mode=Mode::Play;
        }
}

fn bug_movement_system(time: Res<Time>,
    state: Res<State>,
    mut query: Query<(&Bug, &mut Transform)>) {
        if state.mode != Mode::Play {
            return;
        }

    for (bug, mut transform) in &mut query.iter() {
        
        let translation = transform.translation_mut();
        // move the paddle horizontally
        *translation.y_mut() -= time.delta_seconds * bug.speed;
    }
}

fn bug_spawn_system(mut commands: Commands,
    time: Res<Time>, mut state: ResMut<State>,){
    if state.mode != Mode::Play {
        return;
    }
    
    state.timer.tick(time.delta_seconds);

    
    if state.timer.finished {
        let mut rng = thread_rng();
        let x:i32 = rng.gen_range(-5, 5)*BUG_WIDTH as i32 + BUG_WIDTH as i32/2;

        let speed = 40.0 + state.score as f32/1.5;

        state.timer.duration *= 0.98;

        commands
            .spawn(SpriteComponents {
                material: state.bug_texture_handle,
                transform: Transform::from_translation(Vec3::new(x as f32, BUG_POSITION, 0.0)),
                ..Default::default()
            })
            .with(Bug {speed})
            ;
    }
}

fn text_display_system(state: Res<State>, mut query: Query<(&TextMsg, &mut Text, &mut Draw)>) {
    for (msg,mut text, mut draw) in &mut query.iter() {
        match msg {
            TextMsg::Score => {
                text.value = format!("Score: {}", state.score);
            },
            TextMsg::FinalScore => {
                if state.mode == Mode::Stop {
                    text.value = format!("You panic! Final score: {}\nPress <space> to restart!", state.score);
                }
                
                draw.is_visible = state.mode == Mode::Stop;
                
            },
        }
       
    }
}

fn bug_collision_system(
    mut commands: Commands,
    mut state: ResMut<State>,
    audio_output: Res<AudioOutput>,
    mut bug_query: Query<(Entity, &Bug, &Transform, &Sprite)>,
    mut collider_query: Query<(&Collider, &Transform, &Sprite)>,
    
) {
    for (bug_entity, _bug, ball_transform, sprite) in &mut bug_query.iter() {
        let ball_size = sprite.size;
      
        // check collision with walls
        for (collider, transform, sprite) in &mut collider_query.iter() {
            let collision = collide(
                ball_transform.translation(),
                ball_size,
                transform.translation(),
                sprite.size,
            );
            if let Some(_collision) = collision {
                // scorable colliders should be despawned and increment the scoreboard on collision
                match *collider {
                    Collider::Scorable => {
                        audio_output.play(state.bug_sound);
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