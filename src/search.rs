use crate::types::{MatchEntry, Report};
use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;
use std::fs;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn search_pattern(
    pattern: &str,
    path: &str,
    list_only: bool,
    threads: usize,
    json_out: Option<&str>,
) -> Result<()> {
    // Configurar hilos de Rayon
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();

    let regex = Regex::new(pattern)?;

    // Escanear archivos
    let files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
        .map(|e| e.into_path())
        .collect();

    let matches = Arc::new(Mutex::new(Vec::<MatchEntry>::new()));

    files.par_iter().for_each(|file_path| {
        if let Ok(text) = fs::read_to_string(file_path) {
            for (i, line) in text.lines().enumerate() {
                if regex.is_match(line) {
                    let entry = MatchEntry {
                        file: file_path.display().to_string(),
                        line: i + 1,
                        text: line.to_string(),
                    };

                    {
                        let mut guard = matches.lock().unwrap();
                        guard.push(entry.clone());
                    }

                    if !list_only {
                        println!("{}:{}: {}", file_path.display(), i + 1, line);
                    }
                }
            }
        }
    });

    let locked = matches.lock().unwrap();
    let total = locked.len();

    if let Some(path) = json_out {
        let report = Report {
            pattern: pattern.to_string(),
            total_matches: total,
            matches: locked.clone(),
        };

        let json = serde_json::to_string_pretty(&report).unwrap();
        std::fs::write(path, json).expect("No se pudo escribir el JSON");
        println!("Reporte JSON guardado en: {} ({} coincidencias)", path, total);
    } else {
        println!("Total matches: {}", total);
    }

    Ok(())
}
