use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    input::InputSystem,
    prelude::{
        App, CoreSet, Input, IntoSystemConfig, KeyCode, MouseButton, Plugin, Query, Res, ResMut,
    },
};

use bevy_egui::{
    egui::{self},
    EguiContextQuery, EguiSet,
};

use crate::EguiBlockInputState;

pub fn display_stats_ui(mut egui_context: Query<EguiContextQuery>, diagnostics: Res<Diagnostics>) {
    let mut context = egui_context.single_mut();
    egui::Window::new("Stats")
        .title_bar(false)
        .resizable(false)
        .show(context.ctx.get_mut(), |ui| {
            ui.label(format!(
                "FPS: {:.1}",
                match diagnostics
                    .get(FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.average())
                {
                    Some(f) => f,
                    None => 0.0,
                }
            ));
        });
}

pub fn egui_wants_input(
    mut state: ResMut<EguiBlockInputState>,
    mut egui_context: Query<EguiContextQuery>,
) {
    let mut context = egui_context.single_mut();
    let ctx = context.ctx.get_mut();
    state.wants_pointer_input = ctx.wants_pointer_input();
    state.wants_keyboard_input = ctx.wants_keyboard_input();
}

pub fn egui_block_input(
    state: Res<EguiBlockInputState>,
    mut keys: ResMut<Input<KeyCode>>,
    mut mouse_buttons: ResMut<Input<MouseButton>>,
) {
    if state.wants_keyboard_input {
        keys.reset_all();
    }
    if state.wants_pointer_input {
        mouse_buttons.reset_all();
    }
}

pub struct EguiInputBlockerPlugin;

impl Plugin for EguiInputBlockerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EguiBlockInputState>()
            .add_system(
                egui_block_input
                    .after(InputSystem)
                    .in_base_set(CoreSet::PreUpdate),
            )
            .add_system(
                egui_wants_input
                    .after(EguiSet::ProcessOutput)
                    .in_base_set(CoreSet::PostUpdate),
            );
    }
}
