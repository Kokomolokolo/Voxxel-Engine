use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, (
            camera_look, 
            lock_cursor_on_click
        ));
    }
}

#[derive(Component)]
pub struct FpsCamera {
    pub speed: f32,
    pub sensitivity: f32,
}


impl FpsCamera {
    pub fn new() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.001,
        }
    }
}

pub fn setup_camera(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Maus verstecken und einsperren
    if let Ok(mut window) = window_query.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = false;
    }
}

pub fn lock_cursor_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(mut window) = windows.single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }
    }
}

pub fn camera_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &FpsCamera)>,
) {
    for (mut transform, fps_cam) in &mut query {
        for motion in mouse_motion.read() {
            // Yaw = links/rechts (um Y-Achse)
            let yaw = -motion.delta.x * fps_cam.sensitivity;
            
            // Pitch = hoch/runter (um lokale X-Achse)
            let pitch = -motion.delta.y * fps_cam.sensitivity;
            
            // Rotation anwenden
            transform.rotate_y(yaw);
            transform.rotate_local_x(pitch);
        }
    }
}
