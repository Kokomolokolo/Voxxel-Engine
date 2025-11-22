use bevy::prelude::*;
use noise::NoiseFn;

use std::collections::HashMap;

use crate::chunk::*;
use crate::world_gen::*;

#[cfg(target_arch = "wasm32")]
pub const RENDER_DISTACE: i32 = 6;
#[cfg(not(target_arch = "wasm32"))]
pub const RENDER_DISTACE: i32 = 12;

#[derive(Resource)]
pub struct ChunkManager {
    chunks: HashMap<IVec2, Entity>,
    generator: WorldGenerator
} // Hier weiter machen. Die Positionen der Chunks müssen gespeichert werden, und in pos gespeichert werden.
// Sie werden dann über Bevy transform an den richtigen Ort platziert, wenn das das einfachste ist.

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            generator: WorldGenerator::new(12345, 676767)
        }
    }
    pub fn spawn_chunk(
        &mut self,
        pos: IVec2,
        commands: &mut Commands,           // Zum Entities erstellen
        meshes: &mut ResMut<Assets<Mesh>>, // Zum Mesh speichern
        materials: &mut ResMut<Assets<StandardMaterial>>, // Zum Material speichern
        //chunk_query: &Query<&Chunk>,
    ) {
        if self.chunks.contains_key(&pos) { // wenn der Chunk bereits existiert, dann fertig
            return
        }

        let chunk = Chunk::new(pos, &self.generator);
        // Nachbarn sammeln - erstmal raus da ich das gesammte mesh oft neu bauen müsste, habe da keine Lust drauf
        // let mut neighbors: HashMap<IVec2, &Chunk> = HashMap::new();
        // for offset in [IVec2::new(-1, 0), IVec2::new(1, 0), IVec2::new(0, -1), IVec2::new(0, 1)] {
        //     let neighbor_pos = pos + offset;
        //     if let Some(&entity) = self.chunks.get(&neighbor_pos) {
        //         if let Ok(neighbor_chunk) = chunk_query.get(entity) {
        //             neighbors.insert(neighbor_pos, neighbor_chunk);
        //         }
        //     }
        // }
        let mesh = chunk.build_mesh();

        // let mesh_handle = meshes.add(mesh); // Damit verfügbar in Res<Mesh>

        let entity = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                cull_mode: None,  // <- Das ist wichtig!
                ..default()
            })),
            Transform::from_xyz(
                pos.x as f32 * CHUNK_WIDTH as f32,
                0.0,
                pos.y as f32 * CHUNK_WIDTH as f32
            ),
            chunk,
        )).id();
        self.chunks.insert(pos, entity);
    }

    pub fn get_world_block(&self, world_pos: Vec3, chunk_query: &Query<&Chunk>) -> Option<BlockType> { // da möglicherweise der Block nicht geladen ist etc
        let chunk_pos = IVec2::new(
            (world_pos.x / CHUNK_WIDTH as f32).floor() as i32,
            (world_pos.z / CHUNK_WIDTH as f32).floor() as i32,
        );
        let local_x = world_pos.x.rem_euclid(CHUNK_WIDTH as f32) as usize; // rem_euclid ist ähnlich wie %, funktioniert mit neg Zahlen
        let local_y = world_pos.y as usize;
        let local_z = world_pos.z.rem_euclid(CHUNK_WIDTH as f32) as usize;

        if local_y >= CHUNK_HEIGHT || world_pos.y < 0.0 {
            return None;
        }

        let entity = *self.chunks.get(&chunk_pos)?; // da habe ich keine ahnung was das alles macht. mit * wert der poiters entnommen
        let chunk = chunk_query.get(entity).ok()?;
        
        Some(chunk.get_block(local_x, local_y, local_z))
    }

    pub fn get_world_block_mut(&self, world_pos: Vec3, chunk_query: &Query<(&mut Chunk, &Mesh3d)>) -> Option<BlockType> { // da möglicherweise der Block nicht geladen ist etc
        let chunk_pos = IVec2::new(
            (world_pos.x / CHUNK_WIDTH as f32).floor() as i32,
            (world_pos.z / CHUNK_WIDTH as f32).floor() as i32,
        );
        let local_x = world_pos.x.rem_euclid(CHUNK_WIDTH as f32) as usize; // rem_euclid ist ähnlich wie %, funktioniert mit neg Zahlen
        let local_y = world_pos.y as usize;
        let local_z = world_pos.z.rem_euclid(CHUNK_WIDTH as f32) as usize;

        if local_y >= CHUNK_HEIGHT || world_pos.y < 0.0 {
            return None;
        }

        let entity = *self.chunks.get(&chunk_pos)?; // da habe ich keine ahnung was das alles macht. mit * wert der poiters entnommen
        let (chunk, _) = chunk_query.get(entity).ok()?;
        
        Some(chunk.get_block(local_x, local_y, local_z))
    }
    pub fn set_world_block(
        &mut self,
        world_pos: Vec3,
        block_type: BlockType,
        chunk_query: &mut Query<(&mut Chunk, &Mesh3d)>,
        meshes: &mut ResMut<Assets<Mesh>>
    ) {
        let chunk_pos = IVec2::new(
            (world_pos.x / CHUNK_WIDTH as f32).floor() as i32,
            (world_pos.z / CHUNK_WIDTH as f32).floor() as i32,
        );
        let local_x = world_pos.x.rem_euclid(CHUNK_WIDTH as f32) as usize; // rem_euclid ist ähnlich wie %, funktioniert mit neg Zahlen
        let local_y = world_pos.y as usize;
        let local_z = world_pos.z.rem_euclid(CHUNK_WIDTH as f32) as usize;

        if local_y >= CHUNK_HEIGHT || world_pos.y < 0.0 {
            return;
        }
        let Some(&entity) = self.chunks.get(&chunk_pos) else {
            return
        };

        let Ok((mut chunk, mesh_handle)) = chunk_query.get_mut(entity) else {
            return
        };

        chunk.set_block(local_x, local_y, local_z, block_type);

        let new_mesh = chunk.build_mesh();

        if let Some(mesh_asset) = meshes.get_mut(&mesh_handle.0) {
            *mesh_asset = new_mesh;
        }
    }
    pub fn get_biom_at(&self, world_x: i32, world_z: i32) -> f64 {
        self.generator.biom_noise.get([world_x as f64, world_z as f64])
    }
}

use crate::Player;
pub fn update_chunks(
    mut chunk_manager: ResMut<ChunkManager>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    // Camera position in chunk cords umrechnen
    let camera_chunk = IVec2::new(
        (player_transform.translation.x / CHUNK_WIDTH as f32).floor() as i32,
        (player_transform.translation.z / CHUNK_WIDTH as f32).floor() as i32,
    );
    let mut spawned_this_frame = 0;
    const MAX_SPAWNS_PER_FRAME: i32 = 2;
    for x in -RENDER_DISTACE..=RENDER_DISTACE {
        for z in -RENDER_DISTACE..=RENDER_DISTACE {
            if spawned_this_frame > MAX_SPAWNS_PER_FRAME {
                return;
            }
            let chunk_pos = camera_chunk + ivec2(x, z);
            if !chunk_manager.chunks.contains_key(&chunk_pos) {
                chunk_manager.spawn_chunk(chunk_pos, &mut commands, &mut meshes, &mut materials);
                spawned_this_frame += 1;
            }
        }
    }
    despawn_chunks(player_query, chunk_manager, commands);
}

pub fn despawn_chunks(
    player_query: Query<&Transform, With<Player>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut commands: Commands
) {
    let mut to_remove = Vec::new();
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    let camera_chunk = IVec2::new(
        (player_transform.translation.x / CHUNK_WIDTH as f32).floor() as i32,
        (player_transform.translation.z / CHUNK_WIDTH as f32).floor() as i32,
    );
    for chunk in &chunk_manager.chunks {
        let chunk_pos = chunk.0;
        if (chunk_pos.x - camera_chunk.x).abs() > RENDER_DISTACE + 2 
            || (chunk_pos.y - camera_chunk.y).abs() > RENDER_DISTACE + 2 { // +2 für einen buffer, falls bewegung zwischen chunks
            to_remove.push(*chunk_pos); // da chunk.0 keine referenz ist
        }
    }
    for pos in to_remove {
        if let Some(entity) = chunk_manager.chunks.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}