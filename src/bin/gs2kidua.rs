use std::{collections::HashMap, fs, path::Path};

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
    #[arg(short, long, default_value = "kidua")]
    output: String,
}

fn main() {
    let args = Args::parse();

    let input = fs::read_to_string(args.input).unwrap();
    let nade_map = read_gs_json(&input);

    let mut total_count = 0;
    let mut errors = HashMap::new();
    for (map, nades) in nade_map {
        let kidua_results = nades.iter().map(|nade| nade.to_kidua()).collect::<Vec<_>>();
        for result in kidua_results.iter() {
            if let Err(e) = result {
                *errors.entry(e.clone()).or_insert(0) += 1;
            }
        }
        let kidua_nades = kidua_results
            .iter()
            .filter_map(|result| result.clone().ok())
            .collect::<Vec<_>>();
        let count = kidua_nades.len();
        if count == 0 {
            continue;
        }
        total_count += count;
        let kidua_json = json::object! {
            "lineups": kidua_nades,
        };
        let kidua_str = stringify_pretty(kidua_json, 4);
        let dir = Path::new(&args.output);
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(format!("{}.json", map)), kidua_str).unwrap();
        println!("Wrote {} nades for {}", count, map);
    }
    println!("Wrote {} total nades", total_count);
    for (e, count) in errors {
        println!("{}: {}", e, count);
    }
}
