use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};


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
    mut commands: Commands,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Maus verstecken und einsperren
    if let Ok(mut window) = window_query.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(17.0, 32.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        FpsCamera::new(),
    ));
}

pub fn camera_movment(
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FpsCamera)>,
) {
    for (mut transform, fps_cam) in query.iter_mut() {
        // ECS basiert, obwohl es nur eine Kamera gibt
        let mut velocity = Vec3::ZERO;

        let forward = transform.forward();
        let right = transform.right();

        if key.pressed(KeyCode::KeyW) {
            velocity += *forward;
        }
        if key.pressed(KeyCode::KeyA) {
            velocity -= *right;
        }
        if key.pressed(KeyCode::KeyD) {
            velocity += *right;
        }
        if key.pressed(KeyCode::KeyS) {
            velocity -= *forward;
        }

        // Space / Shift
        if key.pressed(KeyCode::Space) {
            velocity.y += 1.0;
        }
        if key.pressed(KeyCode::ShiftLeft) {
            velocity.y -= 1.0;
        }

        transform.translation += velocity * fps_cam.speed * time.delta_secs();
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