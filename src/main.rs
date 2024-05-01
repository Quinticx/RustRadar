mod radar;
mod scan;
mod instance;
mod uniform;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::instance::{CustomMaterialPlugin, InstanceData, InstanceMaterialData};

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(6).build_global().unwrap();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))//(0.52, 0.8, 0.92)))
        .add_plugins((DefaultPlugins, CustomMaterialPlugin))
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, scan::setup_ui)
        .add_systems(Startup, scan::load_scans)
        .add_systems(Update, scan::scan_loaded)
        .add_systems(Update, scan::text_update_system)
        .add_systems(Update, scan::keyboard_input)
        .add_systems(Update, scan::update_filter_system)
        .add_systems(Update, scan::visible_scans)
        .add_systems(Update, scan::move_time)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient: ResMut<AmbientLight>,
) {
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0,25_000.0,0.0).looking_at(Vec3::new(-25_000.0, -25_000.0, -25_000.0), Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    ambient.brightness = 1000.0;

    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(50_000.0)),
        material: materials.add(Color::rgb(65.0/255.0, 152.0/255.0, 10.0/255.0)),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
}