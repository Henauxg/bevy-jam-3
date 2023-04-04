use assets::{
    GameAssets, CLIMBER_RADIUS, HALF_PILLAR_WIDTH, HALF_ROD_WIDTH, PILLAR_HEIGHT, PILLAR_WIDTH,
};
use bevy::{
    app::AppExit,
    core_pipeline::clear_color::ClearColorConfig,
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    pbr::CascadeShadowConfigBuilder,
    prelude::{
        default, info, shape, AmbientLight, App, Assets, BuildChildren, Bundle, Camera3d,
        Camera3dBundle, Color, Commands, Component, DirectionalLight, DirectionalLightBundle,
        Entity, EulerRot, EventReader, EventWriter, Input, KeyCode, Mesh, MouseButton, Name,
        PbrBundle, PluginGroup, Quat, Query, Res, ResMut, SpatialBundle, Transform, Vec2, Vec3,
    },
    window::{PresentMode, Window, WindowCloseRequested, WindowPlugin},
    DefaultPlugins,
};
use bevy_mod_picking::{
    DefaultPickingPlugins, PickableBundle, PickingCameraBundle, PickingEvent, SelectionEvent,
};
use smooth_bevy_cameras::{
    controllers::orbit::{self, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

mod assets;

const WINDOW_TITLE: &str = "Bevy-jam-3";
const CAMERA_CLEAR_COLOR: Color = Color::rgb(0.25, 0.55, 0.92);

pub const GAME_UNIT: f32 = 1.0;
pub const HALF_GAME_UNIT: f32 = GAME_UNIT / 2.;

pub fn exit_on_window_close_system(
    mut app_exit_events: EventWriter<AppExit>,
    mut window_close_requested_events: EventReader<WindowCloseRequested>,
) {
    if !window_close_requested_events.is_empty() {
        app_exit_events.send(AppExit);
        window_close_requested_events.clear();
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(CAMERA_CLEAR_COLOR),
                ..default()
            },
            ..default()
        })
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_translate_sensitivity: Vec2::splat(1.5),
                mouse_rotate_sensitivity: Vec2::splat(0.2),
                ..default()
            },
            Vec3::new(3.0, 7.0, 16.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ))
        .insert(PickingCameraBundle::default());
}

pub fn camera_input_map(
    // egui_input_block_state: Res<EguiBlockInputState>,
    mut events: EventWriter<orbit::ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    _keyboard: Res<Input<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        mouse_wheel_zoom_sensitivity,
        pixels_per_line,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        events.send(orbit::ControlEvent::Orbit(
            mouse_rotate_sensitivity * cursor_delta,
        ));
    }

    if mouse_buttons.pressed(MouseButton::Middle) {
        events.send(orbit::ControlEvent::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta,
        ));
    }

    let mut scalar = 1.0;
    // if !egui_input_block_state.wants_pointer_input {
    for event in mouse_wheel_reader.iter() {
        // scale the event magnitude per pixel or per line
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / pixels_per_line,
        };
        scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
    }
    // }

    events.send(orbit::ControlEvent::Zoom(scalar));
}

fn handle_picking_events(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(SelectionEvent::JustSelected(entity)) => {
                info!("SelectionEvent JustSelected {:?}", entity);
            }
            PickingEvent::Selection(SelectionEvent::JustDeselected(entity)) => {
                info!("SelectionEvent JustDeselected {:?}", entity);
            }
            PickingEvent::Hover(_) => {}
            PickingEvent::Clicked(entity) => {
                info!("Clicked event {:?}", entity);
            }
        }
    }
}

#[derive(Bundle, Default)]
pub struct Levelbundle {
    name: Name,
    spatial: SpatialBundle,
}

impl Levelbundle {
    pub fn new(name: &str) -> Self {
        Self {
            name: Name::from(name),
            ..default()
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Rod {}

#[derive(Component, Clone, Debug)]
pub struct MovableRod {}

#[derive(Component, Clone, Debug)]
pub struct Climber {}

fn spawn_climber(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    x: f32,
    y: f32,
    z: f32,
) -> Entity {
    commands
        .spawn((PbrBundle {
            mesh: assets.climber_mesh.clone(),
            material: assets.climber_mat.clone(),
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },))
        .insert(Climber {})
        // .insert(PickableBundle::default())
        .id()
}

fn spawn_movable_rod(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    x: f32,
    y: f32,
    z: f32,
) -> Entity {
    commands
        .spawn((PbrBundle {
            mesh: assets.movable_rod_mesh.clone(),
            material: assets.static_rod_mat.clone(),
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },))
        .insert(PickableBundle::default())
        .insert(Rod {})
        .insert(MovableRod {})
        .id()
}

fn spawn_static_rod(commands: &mut Commands, assets: &Res<GameAssets>, y: f32, z: f32) -> Entity {
    commands
        .spawn((PbrBundle {
            mesh: assets.static_rod_mesh.clone(),
            material: assets.static_rod_mat.clone(),
            transform: Transform::from_xyz(0.0, y, z),
            ..default()
        },))
        .insert(Rod {})
        .id()
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, assets: Res<GameAssets>) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.2,
    });

    let level_entity = commands.spawn(Levelbundle::new("Test level")).id();

    let dir_light = commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                // Configure the projection to better fit the scene
                shadows_enabled: true,
                color: Color::WHITE,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.6, 0.7, 0.),
                scale: Vec3::new(3., 3., 1.), // TODO Fix: Smaller hides some shadows
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .into(),
            ..default()
        })
        .id();

    let pillar = commands
        .spawn((PbrBundle {
            mesh: meshes.add(shape::Box::new(PILLAR_WIDTH, PILLAR_HEIGHT, PILLAR_WIDTH).into()),
            material: assets.pillar_mat.clone(),
            transform: Transform::from_xyz(0.0, PILLAR_HEIGHT / 2.0, 0.0),
            ..default()
        },))
        .id();

    let s_x_rod_1 = spawn_static_rod(&mut commands, &assets, HALF_ROD_WIDTH, -2.0);
    let s_x_rod_2 = spawn_static_rod(&mut commands, &assets, HALF_ROD_WIDTH + GAME_UNIT, -1.0);

    let x_rod_1 = spawn_movable_rod(&mut commands, &assets, -HALF_GAME_UNIT, 2.0, 2.0);
    let x_rod_2 = spawn_movable_rod(&mut commands, &assets, HALF_GAME_UNIT, 5.0, 1.0);

    let climber_1 = spawn_climber(
        &mut commands,
        &assets,
        HALF_PILLAR_WIDTH + HALF_GAME_UNIT,
        CLIMBER_RADIUS,
        -(HALF_PILLAR_WIDTH + HALF_GAME_UNIT),
    );
    let climber_2 = spawn_climber(
        &mut commands,
        &assets,
        -(HALF_PILLAR_WIDTH + HALF_GAME_UNIT),
        CLIMBER_RADIUS,
        -(HALF_PILLAR_WIDTH + HALF_GAME_UNIT),
    );

    commands
        .entity(level_entity)
        .add_child(dir_light)
        .add_child(pillar)
        .add_child(s_x_rod_1)
        .add_child(s_x_rod_2)
        .add_child(x_rod_1)
        .add_child(x_rod_2)
        .add_child(climber_1)
        .add_child(climber_2);
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE.into(),
            resolution: (800., 600.).into(),
            present_mode: PresentMode::AutoVsync,
            // Tells wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .add_plugin(LookTransformPlugin)
    .add_plugin(OrbitCameraPlugin::new(true))
    .add_plugins(DefaultPickingPlugins);

    app.init_resource::<GameAssets>();

    app.add_startup_system(setup_camera)
        .add_startup_system(setup_scene)
        .add_system(camera_input_map)
        .add_system(handle_picking_events)
        .add_system(exit_on_window_close_system);

    app.run();
}
