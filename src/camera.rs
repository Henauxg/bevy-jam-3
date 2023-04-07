use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::{
        default, Camera, Camera3d, Camera3dBundle, Commands, EventReader, EventWriter, Input,
        KeyCode, MouseButton, Query, Res, Vec2, Vec3,
    },
};
use bevy_mod_picking::PickingCameraBundle;
use smooth_bevy_cameras::controllers::orbit::{self, OrbitCameraBundle, OrbitCameraController};

use crate::{
    assets::DEPRECATED_HALF_PILLAR_HEIGHT, debug::EguiBlockInputState, CAMERA_CLEAR_COLOR,
};

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true, // HDR is required for bloom
                    ..default()
                },
                tonemapping: Tonemapping::TonyMcMapface, // Using a tonemapper that desaturates to white is recommended
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(CAMERA_CLEAR_COLOR),
                    ..default()
                },
                ..default()
            },
            BloomSettings::default(), // Enable bloom for the camera
        ))
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_translate_sensitivity: Vec2::splat(1.5),
                mouse_rotate_sensitivity: Vec2::splat(0.2),
                ..default()
            },
            Vec3::new(3.0, DEPRECATED_HALF_PILLAR_HEIGHT, -20.0),
            Vec3::new(0., DEPRECATED_HALF_PILLAR_HEIGHT, 0.),
            Vec3::Y,
        ))
        .insert(PickingCameraBundle::default());
}

pub fn camera_input_map(
    egui_input_block_state: Option<Res<EguiBlockInputState>>,
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
    let allow_zoom = match egui_input_block_state {
        Some(egui_input_blocker) => !egui_input_blocker.wants_pointer_input,
        None => true,
    };
    if allow_zoom {
        for event in mouse_wheel_reader.iter() {
            // scale the event magnitude per pixel or per line
            let scroll_amount = match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y / pixels_per_line,
            };
            scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
        }
    }

    events.send(orbit::ControlEvent::Zoom(scalar));
}
