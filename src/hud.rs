use bevy::prelude::*;

use crate::chunk_manager::ChunkManager;
use crate::world::WorldGenerator;

// Component für den HUD-Text
#[derive(Component)]
struct HudText;

// Resource zum FPS tracken
#[derive(Resource)]
struct FpsCounter {
    frames: u32,
    timer: f32,
    fps: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            frames: 0,
            timer: 0.0,
            fps: 0.0,
        }
    }
}

// Plugin zum Registrieren
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FpsCounter>()
            .add_systems(Startup, setup_hud)
            .add_systems(Update, (update_fps, update_hud).chain());
    }
}

fn setup_hud(mut commands: Commands) {
    // UI Root Node
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        top: Val::Px(10.0),
        padding: UiRect::all(Val::Px(10.0)),
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new("FPS: 0\nPos: 0.0, 0.0, 0.0"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            HudText,
        ));
    });
}

fn update_fps(
    time: Res<Time>,
    mut fps_counter: ResMut<FpsCounter>,
) {
    fps_counter.frames += 1;
    fps_counter.timer += time.delta_secs();
    
    // Jede Sekunde FPS berechnen
    if fps_counter.timer >= 1.0 {
        fps_counter.fps = fps_counter.frames as f32 / fps_counter.timer;
        fps_counter.frames = 0;
        fps_counter.timer = 0.0;
    }
}

fn update_hud(
    fps_counter: Res<FpsCounter>,
    mut query: Query<&mut Text, With<HudText>>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
) {
    if let Ok(mut text) = query.single_mut() {
        // Kamera-Position holen
        let pos = camera_query
            .single()
            .map(|t| t.translation())
            .unwrap_or(Vec3::ZERO);

        // let biom = generator.get_biom(pos.x.floor() as i32, pos.z.floor() as i32);
        
        // Text updaten
        **text = format!(
            "FPS: {:.0}\nPos: {:.1}, {:.1}, {:.1}\nChunk Pos: {:.0}, {:.0}",
            fps_counter.fps, pos.x, pos.y, pos.z, pos.x / 16.0, pos.z / 16.0, // Da Chunk breite = 16 Blöcke
        );
    }
}

// In main.rs einfügen:
// .add_plugins(HudPlugin)