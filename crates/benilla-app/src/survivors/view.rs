//! Persist the view choice and let the existing WoW controller own third-person play.

use bevy::prelude::*;

use super::view_rules::ViewMode;
use super::{Phase, Run, Selection};

#[derive(Resource)]
pub(super) struct ViewOptions {
    pub(super) mode: ViewMode,
}

impl Default for ViewOptions {
    fn default() -> Self {
        let saved = crate::webenv::var("WOW_SURVIVORS_VIEW").or_else(|| {
            let path = crate::local_state::home()?.join("survivors-view.txt");
            crate::local_state::read_to_string(&path).ok()
        });
        Self {
            mode: saved
                .as_deref()
                .and_then(ViewMode::parse)
                .unwrap_or_default(),
        }
    }
}

impl ViewOptions {
    pub(super) fn toggle(&mut self) {
        self.mode = self.mode.toggled();
        if let Some(path) = crate::local_state::home().map(|p| p.join("survivors-view.txt")) {
            if let Err(error) = crate::local_state::write_atomic(&path, self.mode.saved()) {
                warn!(%error, "survivors: could not save camera choice");
            }
        }
    }
}

/// CaptureMode parks the stock controller; the player UI has its own survival-mode opt-in.
pub(super) fn handoff(
    mut commands: Commands,
    phase: Res<State<Phase>>,
    keys: Res<ButtonInput<KeyCode>>,
    typing: Res<crate::ui_script::UiKeyboardCapture>,
    mut options: ResMut<ViewOptions>,
    capture: Option<Res<crate::run_mode::CaptureMode>>,
    mut camera: ResMut<crate::player::CameraControl>,
    mut player: ResMut<crate::player::Player>,
    mut cursors: Query<&mut bevy::window::CursorOptions, With<bevy::window::PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::F6) && !typing.typing {
        options.toggle();
    }
    let parked = !options.mode.stock_controls(*phase.get() == Phase::Playing);
    if parked && (capture.is_none() || options.is_changed()) {
        if let Ok(mut cursor) = cursors.single_mut() {
            camera.suspend_look(&mut cursor, options.mode == ViewMode::TopDown);
        }
    }
    if parked && capture.is_none() {
        commands.insert_resource(crate::run_mode::CaptureMode);
    } else if !parked && capture.is_some() {
        // Adopt the body's current position after movement in top-down mode or a new run.
        player.reseat = true;
        commands.remove_resource::<crate::run_mode::CaptureMode>();
    }
}

/// A non-combat scene for camera/UI verification. No autopilot, enemies or spell simulation.
#[derive(Resource)]
pub(super) struct Preview;

pub(super) fn preview_from_env() -> bool {
    crate::run_mode::dev_affordances()
        && crate::webenv::var("WOW_SURVIVORS_PREVIEW").as_deref() == Some("1")
}

pub(super) fn enter_preview(
    mut commands: Commands,
    phase: Res<State<Phase>>,
    mut next: ResMut<NextState<Phase>>,
    mut run: ResMut<Run>,
    mut selection: ResMut<Selection>,
    loading: Res<crate::loading_screen::LoadingScreen>,
    heroes: Query<(), With<super::hero::Hero>>,
    mut done: Local<bool>,
) {
    if !*done
        && *phase.get() == Phase::Menu
        && !loading.covering()
        && !heroes.is_empty()
        && super::hud::start_run(&mut run, &mut selection)
    {
        *done = true;
        commands.insert_resource(Preview);
        next.set(Phase::Playing);
        info!("survivors: non-combat view preview ready");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

    #[test]
    fn switching_and_pausing_release_the_cursor_and_reseat_the_controller() {
        let mut app = App::new();
        app.insert_resource(State::new(Phase::Playing))
            .insert_resource(ViewOptions {
                mode: ViewMode::ThirdPerson,
            })
            .insert_resource(crate::run_mode::CaptureMode)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<crate::ui_script::UiKeyboardCapture>()
            .init_resource::<crate::player::CameraControl>()
            .init_resource::<crate::player::Player>()
            .add_systems(Update, handoff);
        let window = app
            .world_mut()
            .spawn((PrimaryWindow, CursorOptions::default()))
            .id();
        app.update();
        assert!(!app
            .world()
            .contains_resource::<crate::run_mode::CaptureMode>());
        assert!(app.world().resource::<crate::player::Player>().reseat);
        {
            let mut cursor = app.world_mut().get_mut::<CursorOptions>(window).unwrap();
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
        app.world_mut().resource_mut::<ViewOptions>().mode = ViewMode::TopDown;
        app.update();
        assert!(app
            .world()
            .contains_resource::<crate::run_mode::CaptureMode>());
        let cursor = app.world().get::<CursorOptions>(window).unwrap();
        assert!(cursor.visible);
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
        assert_eq!(
            app.world()
                .resource::<crate::player::CameraControl>()
                .self_fade(),
            1.0
        );
        app.world_mut().resource_mut::<ViewOptions>().mode = ViewMode::ThirdPerson;
        app.world_mut().insert_resource(State::new(Phase::Paused));
        app.update();
        assert!(app
            .world()
            .contains_resource::<crate::run_mode::CaptureMode>());
        app.world_mut().insert_resource(State::new(Phase::Playing));
        app.world_mut()
            .resource_mut::<crate::player::Player>()
            .reseat = false;
        app.update();
        assert!(!app
            .world()
            .contains_resource::<crate::run_mode::CaptureMode>());
        assert!(app.world().resource::<crate::player::Player>().reseat);
    }
}
