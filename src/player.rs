use bevy::{prelude::*, state::commands};
use crate::camera::FpsCamera;
use crate::chunk::{BlockType, CHUNK_WIDTH, Chunk, ChunkManager};

#[derive(Component)]
pub struct Player {
    pub velocity: Vec3,
    pub grounded: bool,
}

pub fn setup_player(
    mut commands: Commands,
) {
    commands.spawn((
        Player {
            velocity: Vec3::ZERO,
            grounded: false,
        },
        Transform::from_xyz(0.5, 40.0, 0.5)
    ))
    .with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            // Kamera relativ zum Spieler positionieren (z.B. Augenhöhe)
            Transform::from_xyz(0.0, 1.6, 0.0),
            FpsCamera::new(),
        ));
    });
}

pub fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok((mut player, mut player_transform)) = player_query.single_mut() else {
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
    // Y Richtung auf 0: Sonst beim schauen nach oben fliegend
    // direction.y = 0.0; wieder rein sofern der spieler nicht fliegen können soll
    // Damit diagonale Bewegung nicht schneller ist:
    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    // Springen:
    if keys.pressed(KeyCode::Space) {
        direction.y += 2.0; 
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        direction.y -= 2.0; 
    }
    
    let speed = 10.0;
    player.velocity.x = direction.x * speed;
    player.velocity.y = direction.y * speed;
    player.velocity.z = direction.z * speed;

    player_transform.translation += player.velocity * time.delta_secs();
}

pub fn player_physics(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    chunk_manager: Res<ChunkManager>,
    chunk_query: Query<&Chunk>,
    time: Res<Time>,
) {
    let Ok((mut player, mut transform)) = player_query.single_mut() else {
        return;
    };
    
    // Gravität
    player.velocity.y -= 9.81 * time.delta_secs();
    
    // X-Achse separat
    transform.translation.x += player.velocity.x * time.delta_secs();
    resolve_collision_x(&mut transform, &mut player, &chunk_manager, &chunk_query);
    
    // Y-Achse separat
    transform.translation.y += player.velocity.y * time.delta_secs();
    resolve_collision_y(&mut transform, &mut player, &chunk_manager, &chunk_query);
    
    // Z-Achse separat
    transform.translation.z += player.velocity.z * time.delta_secs();
    resolve_collision_z(&mut transform, &mut player, &chunk_manager, &chunk_query);
}

pub fn player_mine_place(
    mouse: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut chunk_query: Query<(&mut Chunk, &Mesh3d)>, // 2 verschiedne, damit alles mesh bezogene in chunks.rs bleibt
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(camera_tranform) = camera_query.single() else {
        return
    };

    // Linksklick: Abbauen
    if mouse.just_pressed(MouseButton::Left) {
        // Ray casting: An der Linie wie die Kamera schaut wird in Intervallen gecheckt, ob sich ein block befindet.
        let ray_start = camera_tranform.translation();
        let ray_dir = camera_tranform.forward();

        let max_dist = 5.0;
        let step_size = 0.05;
        let start_offset = 0.5;

        for i in 0..((max_dist / step_size) as i32) { // jeder block auf dem weg wird gechekct
            let distance = start_offset + (i as f32 * step_size);
            let current_pos = ray_start + ray_dir * distance; // Cool. 
            
            let block_pos = Vec3::new(
                current_pos.x.floor(),
                current_pos.y.floor(),
                current_pos.z.floor(),
            );
            // Ist hier ein Block?
            match chunk_manager.get_world_block_mut(block_pos, &mut chunk_query) {
                Some(BlockType::Air) => {
                    // Kann nicht abgebaut werden, nichts
                }
                Some(block) => {
                    // jeder andere Block kann abgebaut werden:
                    chunk_manager.set_world_block(block_pos,
                        BlockType::Air, 
                        &mut chunk_query, 
                        &mut meshes
                    );
                    break;
                }
                None => {}
            }
        }
    }
    if mouse.just_pressed(MouseButton::Right) {
        // Block wird platziert
        let mut last_air_block: Option<Vec3> = None;
        let mut found_block = false;

        let ray_start = camera_tranform.translation();
        let ray_dir = camera_tranform.forward();

        let max_dist = 5.0;
        let step_size = 0.01;

        for i in 0..((max_dist / step_size) as i32) { // jeder block auf dem weg wird gechekct
            let current_pos = ray_start + ray_dir * (i as f32 * step_size); // Cool. 
            
            let block_pos = Vec3::new(
                current_pos.x.floor() + 0.5,
                current_pos.y.floor() + 0.5,
                current_pos.z.floor() + 0.5,
            );
            // Ist hier ein Block?
            match chunk_manager.get_world_block_mut(block_pos, &mut chunk_query) {
                Some(BlockType::Air) => {
                    last_air_block = Some(block_pos)
                }
                Some(_) => {
                    // etwas anderes als Luft wurde getroffen: Der block muss platziert werden
                    found_block = true;
                    break;
                }
                None => {
                    return // der chunk ist nicht gerendert, es kann nichts platziert werden
                }
            }
        }
        if found_block {
            match last_air_block {
                Some(air_pos) => {
                    chunk_manager.set_world_block(air_pos, 
                        BlockType::Grass, 
                        &mut chunk_query, 
                        &mut meshes
                    );
                }
                None => {
                    // Keine Luft Blöcke gefunden
                }
            }
        }
    }
}














// Hilfsfunktion für X-Kollision
fn resolve_collision_x(
    transform: &mut Transform,
    player: &mut Player,
    chunk_manager: &ChunkManager,
    chunk_query: &Query<&Chunk>,
) {
    let player_half_size = Vec3::new(0.4, 0.9, 0.4);
    
    // ✅ Erst hier berechnen für die Schleifengrenzen
    let player_min_initial = transform.translation - player_half_size;
    let player_max_initial = transform.translation + player_half_size;
    
    let min_block = player_min_initial.floor();
    let max_block = player_max_initial.ceil();
    
    for x in min_block.x as i32 ..= max_block.x as i32 {
        for y in min_block.y as i32 ..= max_block.y as i32 {
            for z in min_block.z as i32 ..= max_block.z as i32 {
                // ✅ NEU BERECHNEN nach jeder möglichen Korrektur!
                let player_min = transform.translation - player_half_size;
                let player_max = transform.translation + player_half_size;
                
                let block_pos = Vec3::new(x as f32, y as f32, z as f32);
                
                if let Some(block_type) = chunk_manager.get_world_block(block_pos, chunk_query) {
                    if block_type != BlockType::Air {
                        let block_min = block_pos - Vec3::splat(0.5);
                        let block_max = block_pos + Vec3::splat(0.5);
                        
                        // Kollisions-Check
                        if player_max.x > block_min.x && player_min.x < block_max.x &&
                           player_max.y > block_min.y && player_min.y < block_max.y &&
                           player_max.z > block_min.z && player_min.z < block_max.z {
                            if player.velocity.x > 0.0 {
                                transform.translation.x = block_min.x - player_half_size.x;
                            } else {
                                transform.translation.x = block_max.x + player_half_size.x;
                            }
                            player.velocity.x = 0.0;
                        }
                    }
                }
            }
        }
    }
}

// Hilfsfunktion für Y-Kollision
fn resolve_collision_y(
    transform: &mut Transform,
    player: &mut Player,
    chunk_manager: &ChunkManager,
    chunk_query: &Query<&Chunk>,
) {
    let player_half_size = Vec3::new(0.4, 0.9, 0.4);
    
    let player_min_initial = transform.translation - player_half_size;
    let player_max_initial = transform.translation + player_half_size;
    
    
    let min_block = player_min_initial.floor();
    let max_block = player_max_initial.ceil();
    
    player.grounded = false;
    
    for x in min_block.x as i32 ..= max_block.x as i32 {
        for y in min_block.y as i32 ..= max_block.y as i32 {
            for z in min_block.z as i32 ..= max_block.z as i32 {
                let player_min = transform.translation - player_half_size;
                let player_max = transform.translation + player_half_size;
                
                let block_pos = Vec3::new(x as f32, y as f32, z as f32);
                
                if let Some(block_type) = chunk_manager.get_world_block(block_pos, chunk_query) {
                    if block_type != BlockType::Air {
                        let block_min = block_pos - Vec3::splat(0.5);
                        let block_max = block_pos + Vec3::splat(0.5);
                        
                        if player_max.x > block_min.x && player_min.x < block_max.x &&
                           player_max.y > block_min.y && player_min.y < block_max.y &&
                           player_max.z > block_min.z && player_min.z < block_max.z {
                            if player.velocity.y > 0.0 {
                                // Nach oben -> Kopf gegen Decke
                                transform.translation.y = block_min.y - player_half_size.y;
                            } else {
                                // Nach unten -> Auf Boden gelandet
                                transform.translation.y = block_max.y + player_half_size.y;
                                player.grounded = true;  // Auf dem Boden!
                            }
                            player.velocity.y = 0.0;
                        }
                    }
                }
            }
        }
    }
}

// Hilfsfunktion für Z-Kollision
fn resolve_collision_z(
    transform: &mut Transform,
    player: &mut Player,
    chunk_manager: &ChunkManager,
    chunk_query: &Query<&Chunk>,
) {
    let player_half_size = Vec3::new(0.4, 0.9, 0.4);
    let player_min_initial = transform.translation - player_half_size;
    let player_max_initial = transform.translation + player_half_size;
    
    
    let min_block = player_min_initial.floor();
    let max_block = player_max_initial.ceil();
    
    for x in min_block.x as i32 ..= max_block.x as i32 {
        for y in min_block.y as i32 ..= max_block.y as i32 {
            for z in min_block.z as i32 ..= max_block.z as i32 {
                let block_pos = Vec3::new(x as f32, y as f32, z as f32);
                let player_min = transform.translation - player_half_size;
                let player_max = transform.translation + player_half_size;
                if let Some(block_type) = chunk_manager.get_world_block(block_pos, chunk_query) {
                    if block_type != BlockType::Air {
                        let block_min = block_pos - Vec3::splat(0.5);
                        let block_max = block_pos + Vec3::splat(0.5);
                        
                        if player_max.x > block_min.x && player_min.x < block_max.x &&
                           player_max.y > block_min.y && player_min.y < block_max.y &&
                           player_max.z > block_min.z && player_min.z < block_max.z {
                            if player.velocity.z > 0.0 {
                                transform.translation.z = block_min.z - player_half_size.z;
                            } else {
                                transform.translation.z = block_max.z + player_half_size.z;
                            }
                            player.velocity.z = 0.0;
                        }
                    }
                }
            }
        }
    }
}