use std::path::Path;
use std::vec::Vec;
use bevy::math::Vec3;
use bevy::prelude::Component;
use chrono::{DateTime, TimeDelta, Utc};
use netcdf::Extents;
use walkdir::WalkDir;
use rayon::prelude::*;

pub trait Radar{
    fn get_gates(&self) -> (std::sync::mpsc::Receiver<Scan>, usize);
}

pub struct AIRRadar{
    
}

#[derive(Component, Clone)]
pub struct ScanMetadata {
    pub name: String,
    pub angular_resolution: f32,
    pub range_resolution: f32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub sweep_index: usize,

    // Aggregate min and max
    pub min: Gate,
    pub max: Gate,
}

pub struct Scan {
    pub meta: ScanMetadata,
    pub gates: Vec<Gate>,
}

#[derive(Debug, Clone)]
pub struct Gate{
    pub reflectivity:f32,
    pub doppler_velocity:f32,
    pub azimuth:f32,
    pub elevation:f32,
    pub range:f32,
}

impl Gate {
    pub fn as_cart(&self) -> Vec3 {
        let y = self.range * self.elevation.sin();
        let z = self.range * self.azimuth.sin();
        let x = self.range * self.azimuth.cos();
        Vec3::new(x, y, z)
    }

    fn max(&self, other: &Self) -> Self {
        Self{
            reflectivity: self.reflectivity.max(other.reflectivity),
            doppler_velocity: self.doppler_velocity.max(other.doppler_velocity),
            azimuth: self.azimuth.max(other.azimuth),
            elevation: self.elevation.max(other.elevation),
            range: self.range.max(other.range),
        }
    }

    fn min(&self, other: &Self) -> Self {
        Self{
            reflectivity: self.reflectivity.min(other.reflectivity),
            doppler_velocity: self.doppler_velocity.min(other.doppler_velocity),
            azimuth: self.azimuth.min(other.azimuth),
            elevation: self.elevation.min(other.elevation),
            range: self.range.min(other.range),
        }
    }
}

impl AIRRadar {
    fn get_gates_from_file(path: &Path) -> Scan {
        let file = netcdf::open(path).unwrap();

        /*
        for var in file.variables() {
            println!("{}", var.name());
            for attr in var.attributes() {
                println!("\t{} = {:?}", attr.name(), attr.value())
            }
            //if var.name() == "sweep_mode" {
                // dbg!(var.name(), var.get_values(Extents::All));
            //}
        }
        */

        let mut buf = [0; 4];
        file.variable("sweep_number").unwrap().get_raw_values(&mut buf, Extents::All).unwrap();
        let sweep = i32::from_le_bytes(buf);

        let mut buf = vec![0; 32];
        let start_time = file.variable("time_coverage_start").unwrap().get_raw_values(&mut buf, Extents::All).unwrap();
        let start_time = String::from_utf8_lossy(&buf);
        let mut start_time = start_time.parse::<DateTime<Utc>>().unwrap();

        let end_time = file.variable("time_coverage_start").unwrap().get_raw_values(&mut buf, Extents::All).unwrap();
        let end_time = String::from_utf8_lossy(&buf);
        let mut end_time = end_time.parse::<DateTime<Utc>>().unwrap();

        if true {
            let sweep_time: TimeDelta = TimeDelta::new(0, 177_777_777).unwrap();
            start_time += sweep_time * sweep;
            end_time = start_time + sweep_time;
        }

        let vel = file.variable("VEL").unwrap();
        let dbz = file.variable("DBZ").unwrap();
        let azimuth = file.variable("azimuth").unwrap();
        let elevation = file.variable("elevation").unwrap();
        let range = file.variable("range").unwrap();

        let dbz_data = dbz.get_values::<i16, _>(..).unwrap();
        let azimuth_data = azimuth.get_values::<f32, _>(..).unwrap();
        let elevation_data = elevation.get_values::<f32, _>(..).unwrap();
        let range_data = range.get_values::<f32, _>(..).unwrap();

        let mut gates = Vec::new();
        for (i, az) in azimuth_data.iter().enumerate() {
            let dbz_data = dbz.get_values::<f32, _>((i, ..)).unwrap();
            let vel_data = vel.get_values::<f32, _>((i, ..)).unwrap();
            for (j, range) in range_data.iter().enumerate() {
                let dbz = dbz_data.get(j).unwrap() * 0.01;
                let vel = vel_data.get(j).unwrap() * 0.01;
                let elevation = elevation_data[i].to_radians();
                gates.push(Gate { azimuth: az.to_radians(), doppler_velocity: vel, range: *range, elevation, reflectivity: dbz});
            }
        }

        let mut min = gates.first().unwrap().clone();
        let mut max = gates.first().unwrap().clone();
        for gate in gates.iter(){
            min = min.min(gate);
            max = max.max(gate);
        }

        return Scan {
            gates,
            meta: ScanMetadata{
                name: path.to_string_lossy().to_string(),
                angular_resolution: (max.azimuth - min.azimuth) / (azimuth_data.len() as f32),
                range_resolution: (max.range - min.range) / (range_data.len() as f32),
                min,
                max,
                start_time,
                end_time,
                sweep_index: sweep as usize,
            }
        };
    }
}

impl Radar for AIRRadar{
    fn get_gates(&self) -> (std::sync::mpsc::Receiver<Scan>, usize) {
        let (tx, rx) = std::sync::mpsc::sync_channel(8);

        let mut all_paths = Vec::new();
        for entry in [
            //"AIR_cfradial/cfrad.20130531_231156_AIR_v1_s1.nc",
            "AIR_cfradial/cfrad*v1_*.nc",
            //"AIR_cfradial/cfrad*v2_*.nc",
            //"AIR_cfradial/cfrad.20130531_231204_AIR_v2_s*.nc",
            //"AIR_cfradial/cfrad.20130531_231211_AIR_v3_s*.nc",
            //"AIR_cfradial/cfrad.20130531_231219_AIR_v4_s*.nc",
            //"AIR_cfradial/cfrad.20130531_231226_AIR_v5_s*.nc",
        ] {
            let glob = glob::glob(entry).unwrap();
            let paths: Vec<_> = glob.collect();
            all_paths.extend(paths.into_iter().filter_map(|path| {
                let path = path.unwrap();
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            }));
        }
        all_paths.sort();

        let count = all_paths.len();
        std::thread::spawn(move || {
                all_paths.par_iter().for_each({
                    let tx = tx.clone();
                    move |path| {
                        let scan = Self::get_gates_from_file(&path);
                        tx.send(scan).unwrap();
                    }
                });
            });
        (rx, count)
    }
}

#[cfg(test)]
mod test{
    use crate::radar::AIRRadar;
    use crate::radar::Radar;
    #[test]
    fn test_air_read(){
        let radar = AIRRadar{};
        let gates = radar.get_gates();
    }
}