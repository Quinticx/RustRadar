use std::ops::{AddAssign, SubAssign};
use std::sync::{Arc, Mutex};
use bevy::asset::{Assets, Handle};
use bevy::input::{ButtonInput, InputPlugin};
use bevy::input::keyboard::Key;
use bevy::math::{Quat, Vec3};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Color, Commands, Component, Cuboid, Entity, KeyCode, Mesh, Query, Res, ResMut, Resource, SpatialBundle, TextBundle, TextStyle, Transform, Visibility, With};
use bevy::render::view::NoFrustumCulling;
use bevy::text::{Text, TextSection};
use bevy::time::Time;
use bevy::ui::{Style, Val};
use bevy::utils::default;
use chrono::{DateTime, TimeDelta, Utc};
use itertools::Position;
use crate::instance::{InstanceData, InstanceMaterialData };
//use crate::instance::{InstanceData, InstanceMaterialData};
use crate::radar;
use crate::scan::ScanType::Reflectivity;
use crate::radar::{Radar, Scan, ScanMetadata};
use crate::uniform::InstanceUniforms;

#[derive(Component, Debug, Eq, PartialEq)]
pub enum ScanType {
    Reflectivity,
    Velocity,
}

#[derive(Resource, Debug)]
pub struct ScanInfo {
    time: Option<DateTime<Utc>>,
    scan_type: ScanType,
    filter: f32,
    step_size: TimeDelta,
    visible_window: TimeDelta,
    time_ratio: f32,
    paused: bool,
    loaded_scans: usize,
}

impl Default for ScanInfo {
    fn default() -> Self {
        Self {
            time: Some("2013-05-31 23:12:04Z".parse::<DateTime<Utc>>().unwrap()),
            filter: 1.0,
            scan_type: ScanType::Reflectivity,
            step_size: TimeDelta::new(8, 0).unwrap(),
            visible_window: TimeDelta::new(8, 0).unwrap(),
            time_ratio: 1.0,
            paused: true,
            loaded_scans: 0,
        }
    }
}

#[derive(Resource)]
pub struct InfoChanged(bool);

pub fn move_time(
    time: Res<Time>,
    mut info: ResMut<ScanInfo>,
) {
    if info.paused {
        return;
    }

    let delta = time.delta_seconds() * info.time_ratio;
    let delta = TimeDelta::new(delta.round() as i64, 0).unwrap();

    info.time.as_mut().map(|time| time.add_assign(delta));
}

pub fn keyboard_input(
    mut change_info: ResMut<InfoChanged>,
    keys: Res<ButtonInput<KeyCode>>,
    mut info: ResMut<ScanInfo>,
) {
    if keys.any_just_pressed([KeyCode::KeyV]) {
        info.scan_type = ScanType::Velocity
    }

    if keys.any_just_pressed([KeyCode::KeyR]) {
        info.scan_type = ScanType::Reflectivity
    }

    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        info.visible_window = TimeDelta::new(0, 177_777_777).unwrap();
    } else {
        info.visible_window = TimeDelta::new(8, 0).unwrap();
    }
    let window = info.visible_window;

    if keys.any_just_pressed([KeyCode::ArrowLeft]) {
        if let Some(time) = info.time.as_mut() {
            *time -= window;
        }
    }

    if keys.any_just_pressed([KeyCode::ArrowRight]) {
        if let Some(time) = info.time.as_mut() {
            *time += window;
        }
    }

    if keys.any_just_pressed([KeyCode::Space]) {
        info.paused = !info.paused;
    }

    if keys.any_just_pressed([KeyCode::ArrowUp]) {
        info.filter += 1.0;
        info.filter = info.filter.min(40.0);
        change_info.0 = true;
    }

    if keys.any_just_pressed([KeyCode::ArrowDown]) {
        info.filter -= 1.0;
        info.filter = info.filter.max(0.0);
        change_info.0 = true;
    }
}

pub fn update_filter_system(
    info: Res<ScanInfo>,
    mut query: Query<(&mut InstanceMaterialData)>,
)
{
    for mut d in query.iter_mut() {
        for i in d.0.iter_mut() {
            i.alpha_pow = info.filter;
        }
    }
}

pub fn visible_scans(
    info: Res<ScanInfo>,
    mut query: Query<(&ScanMetadata, &ScanType, &mut Visibility)>,
){
    let Some(time) = info.time else {
        return;
    };

    for (scan, scan_type, mut visibillity) in query.iter_mut() {
        let window_start = time - info.visible_window;
        if scan.end_time > window_start && scan.end_time <= time && *scan_type == info.scan_type {
            *visibillity = Visibility::Visible
        } else {
            *visibillity = Visibility::Hidden
        }
    }
}

pub fn uniforms(
    mut query: Query<(Entity, &InstanceUniforms)>,
){
    dbg!(query.iter().len());
}

pub fn text_update_system(
    info: Res<ScanInfo>,
    mut query: Query<&mut Text, With<ScanIndexText>>,
) {
    for mut text in &mut query {
        // Update the value of the second section

        if let Some(time) = info.time.as_ref() {
                text.sections[0].value = format!("Time: {} ({})\n", time, if info.paused { String::from("paused") } else { format!("{}x", info.time_ratio)});
                text.sections[1].value = format!("Filter: {}\n", info.filter);
                text.sections[2].value = format!("Scan Type: {:?}\n", info.scan_type);
        };
    }
}

/*
pub fn update_filter_system(
    info: ResMut<ScanInfo>,
    mut change: ResMut<InfoChanged>,
    mut query: Query<(&mut Cuboids, &Scan)>,
) {
    if !change.0 {
        return;
    }
    change.0 = false;

    for (mut cuboids, scan) in query.iter_mut() {
        cuboids.instances.iter_mut().for_each(|cuboid| {
            //let center = (cuboid.maximum + cuboid.minimum) / 2.0;
            let size = cuboid.maximum - cuboid.minimum;
            if size.length() > info.filter * 35.0 {
                cuboid.make_visible();
            } else {
                cuboid.make_invisible();
            }
            //let size = size * info.filter;
            //cuboid.minimum = center - size / 2.0;
            //cuboid.maximum = center + size / 2.0;
        })
    }
}
 */


#[derive(Component)]
pub struct ScanIndexText;

pub fn setup_ui(
    mut commands: Commands,
) {
    commands.insert_resource(ScanInfo::default());
    commands.insert_resource(InfoChanged(true));

    let font_size = 30.0;
    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::from_style(
                TextStyle {
                    font_size,
                    ..default()
                }
            ),
            TextSection::from_style(
                TextStyle {
                    font_size,
                    ..default()
                }
            ),
            TextSection::from_style(
                TextStyle {
                    font_size,
                    ..default()
                }
            ),
            TextSection::from_style(
                TextStyle {
                    font_size,
                    ..default()
                }
            ),
        ]),
        ScanIndexText,
    ));
}

#[derive(Component)]
pub struct ScanLoader {
    rx: Arc<Mutex<std::sync::mpsc::Receiver<Scan>>>,
    total_scans: usize,
}

pub fn load_scans(
    mut commands: Commands,
) {
    let radar = radar::AIRRadar{};
    dbg!("Reading gates");
    let (scans, count) = radar.get_gates();
    commands.spawn(ScanLoader{rx: Arc::new(Mutex::new(scans)), total_scans: count});
}


pub fn scan_loaded(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scan_loader: Query<&mut ScanLoader>,
    mut info: ResMut<ScanInfo>,
    mut query: Query<&mut Text, With<ScanIndexText>>,
) {
    for loader in scan_loader.iter_mut() {
        if info.loaded_scans == 0 {
            query.single_mut().sections[3].value = format!("loading scans {}/{}", info.loaded_scans, loader.total_scans);
        }
        let Ok(scan) = loader.rx.lock().expect("WTF").try_recv() else {
            continue;
        };

        info.loaded_scans += 1;
        if info.loaded_scans == loader.total_scans {
            query.single_mut().sections[3].value = String::from("all scans loaded");
        } else {
            query.single_mut().sections[3].value = format!("loading scans {}/{} ({})", info.loaded_scans, loader.total_scans, scan.meta.name);
        }

        if info.time.is_none() {
            info.time = Some(scan.meta.start_time);
        }

        let gate_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let instance = prepare_reflectivity(&mut commands, &scan);
        commands.spawn((
            gate_mesh.clone(),
            SpatialBundle{
                visibility: Visibility::Hidden,
                transform: Transform::from_xyz(0.0, scan.meta.sweep_index as f32, 0.0),
                ..SpatialBundle::INHERITED_IDENTITY
            },
            InstanceMaterialData(instance),
            InstanceUniforms{
                alpha_power: 5.0,
            },
            NoFrustumCulling,
            ScanType::Reflectivity,
            scan.meta.clone(),
        ));

        let instance = prepare_velocity(&mut commands, &scan);
        commands.spawn((
            gate_mesh.clone(),
            SpatialBundle{
                visibility: Visibility::Hidden,
                ..SpatialBundle::INHERITED_IDENTITY
            },
            InstanceMaterialData(instance),
            InstanceUniforms{
                alpha_power: 5.0,
            },
            NoFrustumCulling,
            ScanType::Velocity,
            scan.meta.clone(),
        ));

        drop(scan);
    }
}

/*
fn color_vel(value: f32) -> Color {
    let colors = [
        ()
    ];
}
 */

fn color(value: f32) -> Color {
    let colors = [
        (Color::BLACK),
        (Color::CYAN),
        (Color::BLUE),
        (Color::MIDNIGHT_BLUE),
        (Color::DARK_GREEN),
        (Color::GREEN),
        (Color::YELLOW),
        (Color::YELLOW_GREEN),
        (Color::ORANGE),
        (Color::ORANGE_RED),
        (Color::RED),
        //(Color::CRIMSON),
        //(Color::MAROON),
        //(Color::PINK),
        //(Color::WHITE),
    ];

    let i = (value / 5.0).floor() as usize;
    colors[i.min(colors.len() - 1)]
}

fn prepare_reflectivity(commands: &mut Commands, scan: &Scan) -> Vec<InstanceData> {
    scan.gates.iter()
        .filter_map(|gate| {
            if gate.reflectivity < 35.0 {
                return None;
            }

            if gate.range < 3000.0 {
                return None;
            }

            let color = color(gate.reflectivity);
            let alpha = (gate.reflectivity / 50.0).min(1.0);//scan.meta.max.reflectivity;
            let size = Vec3::new(
                scan.meta.angular_resolution * gate.range,
                scan.meta.angular_resolution * gate.range,
                scan.meta.range_resolution,
            );

            let t = Transform::from_translation(gate.as_cart()).looking_at(Vec3::ZERO, Vec3::Y).with_scale(size);

            Some(InstanceData{
                scale: gate.range * 0.004,
                position: gate.as_cart(),
                color: color.with_a(alpha).as_linear_rgba_f32(),
                transform: t.compute_matrix().to_cols_array(),
                alpha_pow: 0.0,
            })
        })
        .collect()
}

fn prepare_velocity(commands: &mut Commands, scan: &Scan) -> Vec<InstanceData> {
    scan.gates.iter()
        .filter_map(|gate| {
            if gate.doppler_velocity.abs() < 20.0 {
                return None;
            }

            if gate.range < 3000.0 {
                return None;
            }

            let max_speed = scan.meta.max.doppler_velocity.max(scan.meta.min.doppler_velocity.abs());
            let alpha = gate.doppler_velocity.abs() / max_speed;
            let color = if gate.doppler_velocity < 0.0 {
                Color::rgba(alpha, 0.0, 0.0, alpha)
            } else {
                Color::rgba(0.0, alpha, 0.0, alpha)
            };

            let size = Vec3::new(
                scan.meta.angular_resolution * gate.range,
                scan.meta.angular_resolution * gate.range,
                scan.meta.range_resolution,
            );

            let t = Transform::from_translation(gate.as_cart()).looking_at(Vec3::ZERO, Vec3::Y).with_scale(size);

            Some(InstanceData{
                scale: gate.range * 0.004,
                position: gate.as_cart(),
                color: color.as_linear_rgba_f32(),
                transform: t.compute_matrix().to_cols_array(),
                alpha_pow: 0.0,
            })
        })
        .collect()
}
