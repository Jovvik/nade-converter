use std::{fs, collections::HashMap};

use clap::Parser;
use json::stringify_pretty;
use nade_converter::read_gs_json;

/// Converts skeet-style grenades into primoridal grenades 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input filename
    #[arg(short, long)]
    input: String,

    /// Output directory
    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();

    let input = fs::read_to_string(args.input).unwrap();
    let nade_map = read_gs_json(&input);
    
    let mut total_count = 0;
    for (map, nades) in nade_map {
        let prim_nades = nades
                .into_iter()
                .map(|nade| nade.to_prim())
                .filter_map(Result::ok)
                .enumerate()
                .map(|(i, nade)| (i.to_string(), nade))
                .collect::<HashMap<_, _>>();
        let count = prim_nades.len();
        if count == 0 {
            continue;
        }
        total_count += count;
        let prim_json = stringify_pretty(prim_nades, 4);
        fs::create_dir_all(format!("{}\\{}", args.output, map)).unwrap();
        fs::write(format!("{}\\{}\\nades.json", args.output, map), prim_json).unwrap();
        println!("Wrote {} nades for {}", count, map);
    }
    println!("Got {} primoridal grenades", total_count);
}