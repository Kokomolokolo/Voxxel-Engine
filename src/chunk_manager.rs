// use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::image::ImageSampler;
// use noise::NoiseFn;
use rand::Rng;

use std::collections::HashMap;

use crate::chunk::*;
// use crate::world_gen::*;
use crate::world::*;
use crate::player::Player;

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkManager::new());
        app.add_systems(Startup, setup_block_material);
        app.add_systems(Update, setup_texture_sampling);
        app.add_systems(Update, update_chunks);
    }
}

#[cfg(target_arch = "wasm32")]
pub const RENDER_DISTACE: i32 = 6;
#[cfg(not(target_arch = "wasm32"))]
pub const RENDER_DISTACE: i32 = 12;

#[derive(Resource)]
pub struct ChunkManager {
    chunks: HashMap<IVec2, ChunkEntities>, // 2 Entities für Solid und Liquid
    pub generator: WorldGenerator,
    // dirty_chunks: HashSet<IVec2>,
} // Hier weiter machen. Die Positionen der Chunks müssen gespeichert werden, und in pos gespeichert werden.
// Sie werden dann über Bevy transform an den richtigen Ort platziert, wenn das das einfachste ist.

#[derive(Resource)]
pub struct BlockMaterials {
    pub solid: Handle<StandardMaterial>,
    pub transparent: Handle<StandardMaterial>
}
#[derive(Resource)]
struct BlockTexture {
    handle: Handle<Image>,  // Damit wir später darauf zugreifen können
}

#[derive(Clone, Copy)]
pub struct ChunkEntities {
    pub solid: Entity,
    pub transparent: Entity,
}

// Marker Kompotenten, um solide und Transparente Chucks zu unterscheiden
#[derive(Component)]
pub struct TransparentChunk;

#[derive(Component)]
pub struct  SolidChunk;

impl ChunkManager {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let seed: u32 = rng.gen_range(0..1000);
        Self {
            chunks: HashMap::new(),
            generator: WorldGenerator::new(seed),
            // dirty_chunks: HashSet::new(),
        }
    }
    pub fn spawn_chunk(
        &mut self,
        pos: IVec2,
        commands: &mut Commands,           // Zum Entities erstellen
        meshes: &mut ResMut<Assets<Mesh>>, // Zum Mesh speichern
        // materials: &mut ResMut<Assets<StandardMaterial>>, // Zum Material speichern
        block_material: &Res<BlockMaterials>,
        chunk_query: &Query<&Chunk>,
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
        let solid_mesh = chunk.build_solid_mesh();
        let transparent_mesh = chunk.build_transparent_mesh();

        let base_transform = Transform::from_xyz(
            pos.x as f32 * CHUNK_WIDTH as f32,
            0.0,
            pos.y as f32 * CHUNK_WIDTH as f32
        );

        let solid_entity = commands.spawn((
            Mesh3d(meshes.add(solid_mesh)),
            MeshMaterial3d(block_material.solid.clone()),
            base_transform.clone(),
            chunk.clone(),
            SolidChunk,
        )).id();

        let transparent_entity = commands.spawn((
            Mesh3d(meshes.add(transparent_mesh)),
            MeshMaterial3d(block_material.transparent.clone()),
            base_transform,
            chunk,
            TransparentChunk,
        )).id();
        self.chunks.insert(
            pos, ChunkEntities { solid: solid_entity, transparent: transparent_entity 
        });
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
        let chunk = chunk_query.get(entity.solid).ok()?;
        
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
        let (chunk, _) = chunk_query.get(entity.solid).ok()?;
        
        Some(chunk.get_block(local_x, local_y, local_z))
    }
    pub fn set_world_block(
        &mut self,
        world_pos: Vec3,
        block_type: BlockType,
        chunk_query: &mut Query<(&mut Chunk, &Mesh3d)>,
        transparent_query: &Query<&Mesh3d, With<TransparentChunk>>,
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

        let Ok((mut chunk, solid_mesh_handle)) = chunk_query.get_mut(entity.solid) else {
            return
        };

        chunk.set_block(local_x, local_y, local_z, block_type);

        // Neue Meshes werden erstellt
        let new_solid_mesh = chunk.build_solid_mesh();
        let new_transparent_mesh = chunk.build_transparent_mesh();

        // Meshes werden eigefügt
        if let Some(mesh_asset) = meshes.get_mut(&solid_mesh_handle.0) {
            *mesh_asset = new_solid_mesh;
        }
        if let Ok(transparent_mesh_handle) = transparent_query.get(entity.transparent) {
            if let Some(mesh_asset) = meshes.get_mut(&transparent_mesh_handle.0) {
                *mesh_asset = new_transparent_mesh;
            }
        }
    }
}

pub fn update_chunks(
    mut chunk_manager: ResMut<ChunkManager>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    block_material: Res<BlockMaterials>,
    player_query: Query<&Transform, With<Player>>,
    // mut chunk_query: Query<(&Chunk, &Mesh3d)>,
    chunk_query2: Query<&Chunk>, // WIESO DAS SO IST: ICH WEIß NICHT WIE ES BESSER IST aber naja wenn performance zu bad ist dann ist das so
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    // Camera position in chunk cords umrechnen
    let camera_chunk = IVec2::new(
        (player_transform.translation.x / CHUNK_WIDTH as f32).floor() as i32,
        (player_transform.translation.z / CHUNK_WIDTH as f32).floor() as i32,
    );
    // Lazy Loading für bessere FPS bei vielen neuen Chunks
    let mut spawned_this_frame = 0;
    const MAX_SPAWNS_PER_FRAME: i32 = 2;

    for x in -RENDER_DISTACE..=RENDER_DISTACE {
        for z in -RENDER_DISTACE..=RENDER_DISTACE {
            if spawned_this_frame > MAX_SPAWNS_PER_FRAME {
                break;
            }
            let chunk_pos = camera_chunk + ivec2(x, z);
            if !chunk_manager.chunks.contains_key(&chunk_pos) {
                chunk_manager.spawn_chunk(
                    chunk_pos, 
                    &mut commands, 
                    &mut meshes, 
                    &block_material, 
                    &chunk_query2);
                spawned_this_frame += 1;
            }
        }
        if spawned_this_frame > MAX_SPAWNS_PER_FRAME {
            break;
        }
    }
    // // Dirty Chunks rebuilden - Führt aktuell zu zu großen Lags :/
    // const MAX_REBUILDS_PER_FRAME: usize = 6;
    // let mut rebuilds_couter = 0;

    // // Dirty Chunks in einem Vektor sammeln
    // let chunks_to_rebuild: Vec<_> = chunk_manager.dirty_chunks
    //     .iter()
    //     .copied()
    //     .take(MAX_REBUILDS_PER_FRAME)
    //     .collect();

    // for chunk_pos in chunks_to_rebuild {
    //     chunk_manager.dirty_chunks.remove(&chunk_pos);

    //     // Prüfen ob der Chunk überhaupt noch existiert + Entity holen
    //     let Some(&entity) = chunk_manager.chunks.get(&chunk_pos) else {
    //         continue;
    //     };

    //     let Ok((chunk, mesh_handle)) = chunk_query.get(entity) else {
    //         continue;
    //     };

    //     let new_mesh = chunk.build_mesh();

    //     // Mesh asset updaten
    //     if let Some(mesh_asset) = meshes.get_mut(&mesh_handle.0) {
    //         *mesh_asset = new_mesh;
    //     }
        
    //     rebuilds_couter += 1;
    // }



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
            commands.entity(entity.solid).despawn();
            commands.entity(entity.transparent).despawn();
        }
    }
}

fn setup_block_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let texture_handle: Handle<Image> = asset_server.load("textures/TextureAtlas.png");

    let solid_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        cull_mode: None,
        metallic: 0.0,
        reflectance: 0.5,
        perceptual_roughness: 0.4,
        ..default()
    });

    let transparent_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.7), // 70% Opacity
        cull_mode: None,
        metallic: 0.0,
        reflectance: 0.5,
        perceptual_roughness: 0.2,
        ..default()
    });

    commands.insert_resource(BlockMaterials {
        solid: solid_material,
        transparent: transparent_material,
    });
    commands.insert_resource(BlockTexture { 
        handle: texture_handle  // Hier ohne clone, weil wir es übergeben
    }); 
}
fn setup_texture_sampling(
    mut images: ResMut<Assets<Image>>,      // Zugriff auf alle geladenen Bilder
    block_texture: Res<BlockTexture>,        // Unser Handle von oben
    mut done: Local<bool>,                   // Local = Variable die zwischen Frames gespeichert wird
) {
    // Wenn wir schon fertig sind, nichts mehr tun
    if *done {
        return;
    }
    
    // Versuche die Textur zu holen (gibt None zurück wenn noch nicht geladen)
    if let Some(image) = images.get_mut(&block_texture.handle) {
        // Textur ist geladen! Jetzt Sampler ändern:
        image.sampler = ImageSampler::nearest();
        
        // Merken dass wir fertig sind (damit das nicht jedes Frame läuft)
        *done = true;
    }
    // Wenn None: Textur lädt noch, probieren wir nächstes Frame nochmal
}