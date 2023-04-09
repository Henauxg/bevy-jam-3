use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::{
        default, App, Camera, Camera3d, Camera3dBundle, Commands, EventReader, EventWriter, Input,
        KeyCode, MouseButton, Plugin, Query, Res, Transform, Vec2, Vec3,
    },
    time::Time,
};
use bevy_mod_picking::PickingCameraBundle;
use smooth_bevy_cameras::{
    controllers::orbit::{self, ControlEvent, OrbitCameraBundle, OrbitCameraController},
    LookAngles, LookTransform,
};

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
                tonemapping: Tonemapping::ReinhardLuminance,
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(CAMERA_CLEAR_COLOR),
                    ..default()
                },
                ..default()
            },
            // Enable bloom for the camera
            BloomSettings::default(),
            // FogSettings { // not compatible with grass plugin
            //     color: Color::rgba(0.05, 0.05, 0.05, 1.0),
            //     falloff: FogFalloff::Linear {
            //         start: 5.0,
            //         end: 20.0,
            //     },
            //     ..default()
            // },
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

    // if mouse_buttons.pressed(MouseButton::Middle) {
    //     events.send(orbit::ControlEvent::TranslateTarget(
    //         mouse_translate_sensitivity * cursor_delta,
    //     ));
    // }

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
            scalar = scalar * (1.0 - scroll_amount * mouse_wheel_zoom_sensitivity);
        }
    }

    events.send(orbit::ControlEvent::Zoom(scalar));
}

pub fn control_system(
    time: Res<Time>,
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&OrbitCameraController, &mut LookTransform, &Transform)>,
) {
    // Can only control one camera at a time.
    let (mut transform, scene_transform) =
        if let Some((_, transform, scene_transform)) = cameras.iter_mut().find(|c| c.0.enabled) {
            (transform, scene_transform)
        } else {
            return;
        };

    let mut look_angles = LookAngles::from_vector(-transform.look_direction().unwrap());
    let mut radius_scalar = 1.0;

    let dt = time.delta_seconds();
    for event in events.iter() {
        match event {
            ControlEvent::Orbit(delta) => {
                look_angles.add_yaw(dt * -delta.x);
                look_angles.set_pitch((look_angles.get_pitch() + dt * delta.y).max(0.));
            }
            ControlEvent::TranslateTarget(delta) => {
                let right_dir = scene_transform.rotation * -Vec3::X;
                let up_dir = scene_transform.rotation * Vec3::Y;
                transform.target += dt * delta.x * right_dir + dt * delta.y * up_dir;
            }
            ControlEvent::Zoom(scalar) => {
                radius_scalar *= scalar;
            }
        }
    }

    look_angles.assert_not_looking_up();

    let new_radius = (radius_scalar * transform.radius()).min(30.0).max(8.);
    transform.eye = transform.target + new_radius * look_angles.unit_vector();
}

// #[macro_use]
// mod macros {
//     #[macro_export]
//     macro_rules! define_on_controller_enabled_changed(($ControllerStruct:ty) => {
//         fn on_controller_enabled_changed(
//             mut smoothers: Query<(&mut Smoother, &$ControllerStruct), Changed<$ControllerStruct>>,
//         ) {
//             for (mut smoother, controller) in smoothers.iter_mut() {
//                 smoother.set_enabled(controller.enabled);
//             }
//         }
//     });
// }
// define_on_controller_enabled_changed!(OrbitCameraController);

#[derive(Default)]
pub struct CustomOrbitCameraPlugin;

impl Plugin for CustomOrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            // .add_system(on_controller_enabled_changed.in_base_set(CoreSet::PreUpdate))
            .add_system(control_system)
            .add_event::<ControlEvent>();

        app.add_system(camera_input_map);
    }
}
