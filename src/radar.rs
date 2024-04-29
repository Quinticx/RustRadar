use std::path::Path;
use std::vec::Vec;
use bevy::math::Vec3;
use bevy::prelude::Component;
use walkdir::WalkDir;

pub trait Radar{
    fn get_gates(&self) ->Vec<Scan>;
}

pub struct AIRRadar{
    
}

#[derive(Component)]
pub struct Scan {
    pub angular_resolution: f32,
    pub range_resolution: f32,
    pub gates: Vec<Gate>,
}

#[derive(Debug)]
pub struct Gate{
    pub reflectivity:f32,
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
}

impl AIRRadar {
    fn get_gates_from_file(&self, path: &Path) -> Vec<Gate> {
        let file = netcdf::open(path).unwrap();
        for attr in file.attributes() {
            // dbg!(attr.name(), &attr);
        }

        for var in file.variables() {
            for attr in var.attributes() {
                // dbg!(attr.name(), &attr);
            }
            // dbg!(var.name(), var);
        }

        let dbz = file.variable("DBZ").unwrap();
        let azimuth = file.variable("azimuth").unwrap();
        let elevation = file.variable("elevation").unwrap();
        let range = file.variable("range").unwrap();

        //for attr in dbz.attributes() {
        //    dbg!(attr.name(), attr.value().unwrap());
        //}
        // dbg!(dbz.vartype());
        let dbz_data = dbz.get_values::<i16, _>(..).unwrap();
        let azimuth_data = azimuth.get_values::<f32, _>(..).unwrap();
        let elevation_data = elevation.get_values::<f32, _>(..).unwrap();
        let range_data = range.get_values::<f32, _>(..).unwrap();
        let mut gates = Vec::new();
        for (i, az) in azimuth_data.iter().enumerate() {
            for (j, range) in range_data.iter().enumerate() {
                let value = dbz.get_value::<f32, _>((i, j)).unwrap();
                let elevation = elevation_data[i].to_radians();
                gates.push(Gate { azimuth: az.to_radians(), range: *range, elevation, reflectivity: value * 0.01 });
            }
        }
        return gates;
    }
}

impl Radar for AIRRadar{
    fn get_gates(&self) ->Vec<Scan> {
        let mut scans = Vec::new();

        for entry in [
            "AIR_cfradial/cfrad.20130531_231156_AIR_v1_s*.nc",
            "AIR_cfradial/cfrad.20130531_231204_AIR_v2_s*.nc",
            "AIR_cfradial/cfrad.20130531_231211_AIR_v3_s*.nc",
            "AIR_cfradial/cfrad.20130531_231219_AIR_v4_s*.nc",
            "AIR_cfradial/cfrad.20130531_231226_AIR_v5_s*.nc",
        ] {
            let mut gates = Vec::new();
            for entry in glob::glob(entry).unwrap() {
                let entry = entry.unwrap();
                if entry.is_file() {
                    gates.extend(self.get_gates_from_file(&entry));
                }
            }
            let angular_resolution = (16.517744_f32-16.002102_f32).to_radians();
            let range_resolution = (gates.get(0).unwrap().range-gates.get(1).unwrap().range).abs();
            scans.push(Scan {
                angular_resolution,
                range_resolution,
                gates
            });
        }
        scans
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
        // dbg! (gates);
    }
}