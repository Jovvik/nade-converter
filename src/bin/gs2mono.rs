use std::{collections::HashMap, fs};

use clap::Parser;
use json::{stringify_pretty, JsonValue};
use nade_converter::read_gs_json;
use phf::{phf_map, Map};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input filename
    #[arg(short, long)]
    input: String,

    /// Output filename
    #[arg(short, long)]
    output: String,
}

const ITEMINDEX_TO_MONO: Map<&'static str, &'static str> = phf_map!(
    "weapon_molotov" => "fire",
    "weapon_hegrenade" => "he",
);

fn main() {
    let args = Args::parse();

    let input = fs::read_to_string(args.input).unwrap();
    let nade_map = read_gs_json(&input);
    let mut mono_map = HashMap::new();
    for (map, nades) in nade_map {
        let mut weapon_map: HashMap<String, Vec<JsonValue>> = HashMap::new();
        for nade in nades {
            let mono_weapon_name = ITEMINDEX_TO_MONO.get(&nade.weapon);
            if None == mono_weapon_name {
                // println!("Unknown weapon name {}", nade.weapon);
                continue;
            }
            let mono_weapon_name = mono_weapon_name.unwrap();
            match nade.to_mono() {
                Ok(nade_json) => {
                    weapon_map
                        .entry(mono_weapon_name.to_owned().to_owned())
                        .or_insert_with(Vec::new)
                        .push(nade_json);
                }
                Err(e) => println!("Error coverting to mono: {}", e),
            }
        }
        mono_map.insert(map, weapon_map);
    }
    let total_mono_nades = mono_map
        .values()
        .map(|v| v.values().map(|v| v.len()).sum::<usize>())
        .sum::<usize>();
    println!("Total mono nades: {}", total_mono_nades);
    // serialize mono_map with json and write to output
    let mono_json = stringify_pretty(mono_map, 4);
    fs::write(args.output, mono_json).unwrap();
}
