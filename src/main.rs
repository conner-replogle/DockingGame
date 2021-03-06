use bevy::{core::FixedTimestep, ecs::schedule::SystemSet, prelude::*, render::camera::{Camera3d, Camera2d}, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}};
use rand::Rng;
use bevy::sprite::Rect;
use bevy_rapier2d::prelude::*;
//use num_traits::float::FloatConst;
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<Game>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_state(GameState::Playing)
        .add_startup_system(setup_cameras)
.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_player)
                .with_system(follow_player)
                .with_system(spaceship_animation)
                .with_system(debug_ui),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown))
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(gameover_keyboard))
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(teardown))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(5.0))
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
#[derive(Bundle)]
struct Player{
    space_ship: SpaceShip,
    controller: PlayerMoveable,
}




#[derive(Component)]
struct SpaceShip;
#[derive(Component)]
struct PlayerMoveable;
enum SpaceShipMode{
    ThrustingForward,
    Standby,

}
#[derive(Component)]
struct SpaceShipAnimation{
    mode: SpaceShipMode
}
#[derive(Default)]
struct Game {
    score: i32
}


fn setup_cameras(mut commands: Commands, mut game: ResMut<Game>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>,mut texture_atlases: ResMut<Assets<TextureAtlas>>,) {
    // reset the game state
    game.score = 0;

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            range: 30.0,
            ..default()
        },
        ..default()
    });
    let texture_handle = asset_server.load("sprites/spaceship.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(128.0, 120.0), 8, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::CRIMSON,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(200.0,200.0,0.0),
                scale: Vec3::new(100.0, 100.0, 10.0),
                ..default()
            },
            ..default()
        });
    commands
        .spawn()
        .insert(GravityScale(0.0))
        .insert(RigidBody::Dynamic)
        .insert(Velocity {
            linvel: Vec2::new(0.0, 0.0),
            angvel: 0.0
        })
        .insert(Collider::ball(50.0))
        .insert(SpaceShipAnimation{
            mode: SpaceShipMode::Standby,
        })
        .insert(ExternalForce{
            force: Vec2::ZERO,
            torque: 0.0,
        }).insert(MassProperties {
            local_center_of_mass: Vec2::new(10.0, 0.0),
            mass: 100.0,
            principal_inertia: 1.0,
        })
        .insert(Damping { linear_damping: 0.0, angular_damping: 0.0 })
        .insert_bundle(Player {
            space_ship: SpaceShip{
                
            },
            controller: PlayerMoveable,
        }).insert_bundle(SpriteSheetBundle{
            transform: Transform{
                translation: Vec3::new(20., 20., 1.),
                ..default()
            },
            sprite: TextureAtlasSprite{
                index: 1,
                ..default()
            },
            texture_atlas: texture_atlas_handle,
            ..default()
        });

    // scoreboard
    commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            "ALPHA",
            TextStyle {
                font: asset_server.load("fonts/retro_gaming_font.ttf"),
                font_size: 20.0,
                color: Color::rgb(1., 1., 1.),
            },
            Default::default(),
        ),
        style: Style {
            position_type: PositionType::Absolute,
            position: bevy::prelude::Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

// remove all entities that are not a camera
fn teardown(mut commands: Commands, entities: Query<Entity, Without<Camera>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn follow_player(time: Res<Time>,player_query: Query<& Transform,With<PlayerMoveable>>,mut camera: Query<&mut Transform,(With<Camera2d>,Without<PlayerMoveable>)>){
    let player = player_query.get_single().unwrap();
    let mut camera = camera.get_single_mut().unwrap();
    let cam_x = camera.translation.x;
    let cam_y = camera.translation.y;

    let mut x = player.translation.x;
    let mut y = player.translation.y;

    let del_x = x - cam_x;
    let del_y = y - cam_y;
    let distance = f32::sqrt((del_x * del_x) + (del_y * del_y));
    let new_x:f32 ;
    let new_y: f32;
    //println!("distance {}",distance);
    if distance > 50.{
        new_x = del_x *((distance-50.) / 100.) *time.delta_seconds();
        new_y = del_y *((distance-50.) / 100.) *time.delta_seconds();
        camera.translation = Vec3::new(cam_x + new_x,cam_y + new_y,camera.translation.z);
    

    }

    
}

// control the game character
fn move_player(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut ext_force: Query<(&mut ExternalForce,&mut Damping,&mut Transform),With<PlayerMoveable>>,
    mut spaceship_animation: Query<&mut SpaceShipAnimation,With<PlayerMoveable>>,
    time: Res<Time>,
) {
    const SPEED:f32 = 50.0;
    const TURNING_SPEED:f32 = 0.075;
    const BRAKE_INTESITY:f32 = 25.;
    let mut acceleration = 0.0;
    let mut angle = 0.0;
    let mut spaceship_anim = spaceship_animation.get_single_mut().unwrap();
    spaceship_anim.mode = SpaceShipMode::Standby;
    if keyboard_input.pressed(KeyCode::W){
        acceleration -= SPEED; 
        spaceship_anim.mode = SpaceShipMode::ThrustingForward;
    }
    if keyboard_input.pressed(KeyCode::S){
        acceleration += BRAKE_INTESITY; 
    }
    
    if keyboard_input.pressed(KeyCode::D){
        angle -= TURNING_SPEED;
    }
    if keyboard_input.pressed(KeyCode::A){
        angle += TURNING_SPEED;
    }
    ext_force.for_each_mut(|(mut force,mut damping,tra)| {

        force.torque = angle;
        let norm = tra.rotation.normalize();
        let euler = norm.to_euler(EulerRot::XYZ);
        let mut z_deg = euler.2 * 180.0 / std::f64::consts::PI as f32;
        if z_deg < 0.0{
            z_deg+= 360.0;

        }
        let z_rad = z_deg * (std::f64::consts::PI as f32/180.0);
        let y_cord = z_rad.sin()*acceleration;
        let x_cord = z_rad.cos()*acceleration;
        //println!("Z: {}",z_rad); 
        force.force = Vec2::new(x_cord,y_cord);
        
    })
    
}
fn spaceship_animation(
    mut spaceship_animation: Query<&mut SpaceShipAnimation,With<PlayerMoveable>>,
    mut texture_atlas: Query<&mut TextureAtlasSprite,With<PlayerMoveable>>,){
        let mut text_atlas = texture_atlas.get_single_mut().unwrap();
        let spaceship = spaceship_animation.get_single_mut().unwrap();
        match spaceship.mode{
            SpaceShipMode::ThrustingForward => {
                if text_atlas.index < 5{
                    text_atlas.index+=1;
                }else{
                    if text_atlas.index < 7{
                        text_atlas.index+=1;
                    }else{
                        text_atlas.index = 5;
                    }

                }
            },
            SpaceShipMode::Standby => {
                text_atlas.index = 1;
            },
        }
}




// update the score displayed during the game
fn debug_ui( mut query: Query<&mut Text>, player_q: Query<(&Transform,&Velocity,&ExternalForce),With<PlayerMoveable>>) {
    let mut text = query.single_mut();
    let (transform,velocity,ext_force) = player_q.single();
    text.sections[0].value = format!("Position: {} \n Velocity: (LV:{},AV:{}) \n ExternalForce: (F:{},T:{})", transform.translation.round(),velocity.linvel.round(),velocity.angvel.round(),ext_force.force.round(),ext_force.torque.round());
}

// restart the game when pressing spacebar
fn gameover_keyboard(mut state: ResMut<State<GameState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.set(GameState::Playing).unwrap();
    }
}


