use bevy::asset::Assets;
use bevy::input::{ButtonInput, InputPlugin};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Color, Commands, Component, Cuboid, Event, EventReader, EventWriter, KeyCode, Mesh, Query, Res, ResMut, Resource, SpatialBundle, TextBundle, TextStyle, Visibility, With};
use bevy::render::view::NoFrustumCulling;
use bevy::text::{Text, TextSection};
use bevy::ui::{Style, Val};
use bevy::utils::default;
use itertools::Position;
use crate::instance::{InstanceData, InstanceMaterialData};
//use crate::instance::{InstanceData, InstanceMaterialData};
use crate::radar;
use crate::scan::ScanType::Reflectivity;
use crate::radar::{Radar, Scan};

#[derive(Component, Debug)]
pub struct ScanNumber(pub usize);

#[derive(Component, Debug, Eq, PartialEq)]
pub enum ScanType {
    Reflectivity,
}

#[derive(Resource, Debug)]
pub struct ScanInfo {
    number: usize,
    scan_type: ScanType,
    filter: f32,
}

impl Default for ScanInfo {
    fn default() -> Self {
        Self {
            number: 0,
            filter: 1.0,
            scan_type: ScanType::Reflectivity,
        }
    }
}

#[derive(Resource)]
pub struct InfoChanged(bool);

pub fn keyboard_input(
    mut change_info: ResMut<InfoChanged>,
    keys: Res<ButtonInput<KeyCode>>,
    mut info: ResMut<ScanInfo>,
) {
    if keys.any_just_pressed([KeyCode::ArrowLeft]) {
        info.number -= 1
    }

    if keys.any_just_pressed([KeyCode::ArrowRight]) {
        info.number += 1
    }

    if keys.any_just_pressed([KeyCode::ArrowUp]) {
        info.filter += 0.05;
        info.filter = info.filter.min(1.0);
        change_info.0 = true;
    }

    if keys.any_just_pressed([KeyCode::ArrowDown]) {
        info.filter -= 0.05;
        info.filter = info.filter.max(0.0);
        change_info.0 = true;
    }
}

pub fn visible_scans(
    info: Res<ScanInfo>,
    mut query: Query<(&ScanNumber, &ScanType, &mut Visibility)>,
){
    for (number, scan_type, mut visibillity) in query.iter_mut() {
        if number.0 == info.number && *scan_type == info.scan_type {
            *visibillity = Visibility::Visible
        } else {
            *visibillity = Visibility::Hidden
        }
    }
}

pub fn text_update_system(
    info: Res<ScanInfo>,
    mut query: Query<&mut Text, With<ScanIndexText>>,
) {
    for mut text in &mut query {
        // Update the value of the second section
        text.sections[0].value = format!("Scan number: {}", info.number);
        text.sections[1].value = format!("Not Attenuation: {}", info.filter);
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

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(ScanInfo::default());
    commands.insert_resource(InfoChanged(true));

    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::from_style(
                TextStyle {
                    font_size: 60.0,
                    ..default()
                }
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 60.0,
                    ..default()
                }
            ),
        ]),
        ScanIndexText,
    ));

    let radar = radar::AIRRadar{};
    dbg!("Reading gates");
    let scans = radar.get_gates();
    dbg!("done");

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

    let gate_mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
    for (i, scan) in scans.into_iter().enumerate() {
        let gates = scan.gates.iter()
            .filter_map(|gate| {
                if gate.reflectivity < 25.0 {
                    return None;
                }

                if gate.range < 3000.0 {
                    return None;
                }

                let i = (gate.reflectivity / 5.0).floor() as usize;
                let color = colors[i.min(colors.len() - 1)];

                let alpha = (gate.reflectivity / (colors.len() as f32 * 5.0)).min(1.0).powf(5.0);
                // let alpha = 0.5;//((gate.reflectivity - 30.0) / 30.0).min(1.0);

                //let alpha = gate.reflectivity;
                //let max_alpha = ((5 * colors.len()) as f32);
                let size = scan.range_resolution * gate.range;

                let color = color.with_a(alpha);

                Some(InstanceData{
                    scale: gate.range * 0.008,
                    position: gate.as_cart(),
                    color: color.as_linear_rgba_f32(),
                })
            })
            .collect();

        commands.spawn((
            gate_mesh.clone(),
            SpatialBundle{
                visibility: Visibility::Hidden,
                ..SpatialBundle::INHERITED_IDENTITY
            },
            InstanceMaterialData(gates),
            NoFrustumCulling,
            ScanType::Reflectivity,
            ScanNumber(i),
        ));
    }

}