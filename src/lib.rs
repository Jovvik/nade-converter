use std::collections::HashMap;

use assert_float_eq::*;
use json::JsonValue;
use phf::{phf_map, Map};

#[derive(Debug, Clone)]
pub struct Grenade {
    from: String,
    to: String,
    pub weapon: String, // pub for serialization to mono format
    x: f32,
    y: f32,
    z: f32,
    // position visibility
    yaw: f32,
    pitch: f32,
    description: String,
    duck: bool,
    // tickrate
    // approach_accurate
    strength: f32,
    // fov
    jump: bool,
    run: i32,
    pub run_yaw: f32,
    run_speed: bool,
    recovery_yaw: f32,
    recovery_jump: bool,
    delay: u32,
    // destroy
    // target
}

static YAW_TO_DIRECTION: Map<i32, &'static str> = phf_map! {
    0i32 => "f",
    90i32 => "r",
    180i32 => "b",
    -90i32 => "l",
    -180i32 => "b"
};

impl Grenade {
    fn new() -> Grenade {
        Grenade {
            from: String::from(""),
            to: String::from(""),
            weapon: String::from(""),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            description: String::from(""),
            duck: false,
            strength: 1.0,
            jump: false,
            run: 0,
            run_yaw: 0.0,
            run_speed: false,
            recovery_yaw: 0.0,
            recovery_jump: false,
            delay: 0,
        }
    }

    pub fn from_gs_json(json: &JsonValue) -> Result<Grenade, String> {
        let mut grenade = Grenade::new();
        grenade.from = json["name"][0].as_str().ok_or("No from")?.to_owned();
        grenade.to = json["name"][1].as_str().ok_or("No to")?.to_owned();
        grenade.description = json["description"].as_str().unwrap_or("").to_owned();
        grenade.weapon = json["weapon"].as_str().ok_or("No weapon")?.to_owned();
        grenade.x = json["position"][0].as_f32().ok_or("No x")?.to_owned();
        grenade.y = json["position"][1].as_f32().ok_or("No y")?.to_owned();
        grenade.z = json["position"][2].as_f32().ok_or("No z")?.to_owned();
        grenade.yaw = json["viewangles"][1].as_f32().ok_or("No yaw")?.to_owned();
        grenade.pitch = json["viewangles"][0].as_f32().ok_or("No pitch")?.to_owned();
        grenade.duck = json["duck"].as_bool().unwrap_or(false);
        grenade.strength = json["grenade"]["strength"].as_f32().unwrap_or(1.0);
        grenade.jump = json["grenade"]["jump"].as_bool().unwrap_or(false);
        let run = json["grenade"]["run"].as_f32().unwrap_or(0.0);
        if run.fract().abs() >= 0.0000001 {
            return Err("Run is not an integer".to_owned());
        } else if run.is_sign_negative() {
            return Err("Run is negative".to_owned());
        } else {
            grenade.run = run as i32;
        }
        grenade.run_yaw = json["grenade"]["run_yaw"].as_f32().unwrap_or(0.0);
        grenade.run_speed = json["grenade"]["run_speed"].as_bool().unwrap_or(false);
        grenade.recovery_yaw = json["grenade"]["recovery_yaw"]
            .as_f32()
            .unwrap_or(grenade.run_yaw - 180.0);
        grenade.recovery_jump = json["grenade"]["recovery_jump"].as_bool().unwrap_or(false);
        let delay = json["grenade"]["delay"].as_f32().unwrap_or(0.0);
        if delay.fract().abs() >= 0.0000001 {
            return Err("Delay is not an integer".to_owned());
        } else if delay.is_sign_negative() {
            return Err("Delay is negative".to_owned());
        } else {
            grenade.delay = delay as u32;
        }

        Ok(grenade)
    }

    pub fn to_mono(&self) -> Result<JsonValue, String> {
        if self.run_speed {
            return Err("Run speed is not supported".to_owned());
        }
        if self.run_yaw.fract().abs() >= 0.000001 {
            return Err(format!("Run yaw is non-integer: {}", self.run_yaw));
        }
        let run_direction = self.yaw_to_direction(self.run_yaw)?;
        let recovery_direction = self.yaw_to_direction(self.recovery_yaw)?;
        let mut m = run_direction.to_owned();
        let mut r = recovery_direction.to_owned();
        if self.jump {
            m.push('j');
        }
        if self.duck {
            m.push('d');
        }
        if self.recovery_jump {
            r.push('j');
        }
        let mut name = self.to.clone();
        if !self.description.is_empty() {
            name.push_str(&format!(" ({})", self.description));
        }
        Ok(json::object! {
            n: name,
            x: self.x,
            y: self.y,
            z: self.z,
            yaw: self.yaw,
            pitch: self.pitch,
            st: self.strength as i32 * 2,
            tr: self.run as f32 / 64.0,
            jtt: self.delay as f32 / 64.0,
            rt: if r == "f" { 0.0 } else { 0.5 },
            m: m,
            r: r,
        })
    }

    fn yaw_to_direction(&self, yaw: f32) -> Result<&str, String> {
        Ok(*match YAW_TO_DIRECTION.get(&(yaw as i32)) {
            None => {
                return Err(format!("Unknown run direction: {}", yaw));
            }
            Some(dir) => dir,
        })
    }
}

pub fn read_gs_json(data: &str) -> HashMap<String, Vec<Grenade>> {
    let in_json = json::parse(data).unwrap();
    let mut nades_map: HashMap<String, Vec<Grenade>> = HashMap::new();
    for (map, nades) in in_json.entries() {
        nades_map.insert(map.to_owned(), vec![]);
        for nade in nades.members() {
            let grenade = Grenade::from_gs_json(nade);
            match grenade {
                Ok(grenade) => {
                    nades_map.get_mut(map).unwrap().push(grenade);
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        println!("Map: {}, nades: {}", map, nades_map[map].len());
    }
    let total_nades: usize = nades_map.values().map(|v| v.len()).sum();
    println!("Nades read: {}", total_nades);
    nades_map
}

#[cfg(test)]
mod tests {
    use assert_float_eq::assert_f32_near;
    use json::object;

    pub use super::*;

    #[test]
    fn parsing() {
        let test_gs_nade = object! {
                    "name": ["T Roof", "Scaffolding Box"], // array of from and to, alternatively a single string
            "description": "Jump on the left box for a good one-way", //optionally, a description can be given
            "weapon": "weapon_smokegrenade", // weapon console name
            "position": [691.63653564453, -1130.1051025391, -127.96875], // origin
            "position_visibility": [-44, 0, 0], // offset to origin for world vischeck, defaults to [0, 0, 0]
            "viewangles": [-1.8710323572159, -136.26739501953], // pitch, yaw
            "duck": true, // true = have to be fully ducked, defaults to false
            "tickrate": 128, // number: all tickrates supported, array: the only tickrates supported
            "approach_accurate": true, // full speed movement during approach, auto-checked by default
            "grenade": {
                "strength": 0.5, // required m_flThrowStrength to autothrow, 1=left, 0.5 = right+left, 0 = right
                "fov": 0.3, // have to be in this fov to autothrow
                "jump": true, // jumpthrow at the end of running
                "run": 12, // run duration in seconds/64
                "run_yaw": 90, // offset to viewangles for move yaw
                "run_speed": true, // hold IN_SPEED (shift) during pre-throw run, defaults to false
                "recovery_yaw": 90, // yaw for movement after throw, only rage aimbot mode. Defaults to run_yaw-180
                "delay": 5 // delay before throwing, useful for getting the max height in a jumpthrow. Defaults to 0
            },
            "destroy": { // a breakable world object has to be destroyed before autothrowing / playback
                "start": [392.701141, -1442.725342, 1936.63842], // trace_line starts from here
                "end": [232.03129134004, -1425.9891813532, 1899.5775623479], // trace_line ends here
                "text": "Break the left window" // text to add if trace_line hit something
            },
            "target": [-19.584310531616, -1810.5485839844, -110.97956085205] // grenade / shot will land here
        };
        let nade = Grenade::from_gs_json(&test_gs_nade);
        assert!(nade.is_ok());
        let nade = nade.unwrap();
        assert_eq!(nade.from, "T Roof");
        assert_eq!(nade.to, "Scaffolding Box");
        assert_eq!(nade.description, "Jump on the left box for a good one-way");
        assert_eq!(nade.weapon, "weapon_smokegrenade");
        assert_f32_near!(nade.x, 691.636_54);
        assert_f32_near!(nade.y, -1_130.105_1);
        assert_f32_near!(nade.z, -127.96875);
        assert_f32_near!(nade.pitch, -1.871_032_4);
        assert_f32_near!(nade.yaw, -136.267_4);
        assert!(nade.duck);
        assert_f32_near!(nade.strength, 0.5);
        assert!(nade.jump);
        assert_eq!(nade.run, 12);
        assert_f32_near!(nade.run_yaw, 90.0);
        assert_eq!(nade.delay, 5);
    }
}
