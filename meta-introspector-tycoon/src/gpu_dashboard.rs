use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "🌌 Meta-Introspector Tycoon - GPU Dashboard 🌌".into(),
                resolution: bevy::window::WindowResolution::new(1920.0, 1080.0),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_factories)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 8.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.0, 1.0, 0.0),
            illuminance: 8000.0,
            ..default()
        },
        ..default()
    });

    // Factory cubes
    for i in 0..5 {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.0, 1.0, 0.0),
                    emissive: Color::rgb(0.0, 0.5, 0.0),
                    ..default()
                }),
                transform: Transform::from_xyz(i as f32 * 4.0 - 8.0, 0.0, 0.0),
                ..default()
            },
            Factory { level: 1 },
        ));
    }
}

#[derive(Component)]
struct Factory {
    level: u32,
}

fn animate_factories(time: Res<Time>, mut query: Query<&mut Transform, With<Factory>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_y(0.01);
        let scale = 1.0 + (time.elapsed_seconds().sin() * 0.1);
        transform.scale = Vec3::splat(scale);
    }
}
