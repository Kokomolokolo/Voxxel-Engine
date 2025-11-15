use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub velocity: Vec3,
    pub grounded: bool,
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>
) {
    let Ok((mut player, plyr_transform)) = player_query.single_mut() else {
        return;
    };
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let forward = camera_transform.forward();
    let right = camera_transform.right();

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction += *forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= *right;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= *forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += *right;
    }
}