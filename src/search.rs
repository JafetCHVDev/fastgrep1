use crate::types::{MatchEntry, Report};
use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;
use std::fs;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

/// Patrones inteligentes PRO
pub fn smart_pattern(kind: &str) -> Option<String> {
    let patterns = [
        ("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"),
        ("ipv4",  r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
        ("url",   r#"https?://[^\s"']+"#),
        ("jwt",   r"[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+"),
        ("token", r"(?:ghp|gho|github_pat|ya29)\w{20,}"),
        ("uuid",  r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"),
        ("creditcard", r"\b(?:\d[ -]*?){13,16}\b"),
        ("hex",   r"\b[A-Fa-f0-9]{8,}\b"),
    ];

    patterns
        .iter()
        .find(|(name, _)| *name == kind)
        .map(|(_, regex)| regex.to_string())
}

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

    // Si el patrón comienza con "smart:" usar motor inteligente
    let regex_text = if let Some(stripped) = pattern.strip_prefix("smart:") {
        smart_pattern(stripped)
            .expect("Tipo de patrón inteligente no reconocido (email, ipv4, url, ...)")
    } else {
        pattern.to_string()
    };

    let regex = Regex::new(&regex_text)?;

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
                if let Some(mat) = regex.find(line) {
                    let entry = MatchEntry {
                        file: file_path.display().to_string(),
                        line: i + 1,
                        matched_text: mat.as_str().to_string(),
                    };

                    {
                        let mut guard = matches.lock().unwrap();
                        guard.push(entry);
                    }

                    // Salida normal (la tabla se genera en main.rs)
                    if !list_only {
                        let highlighted = line.replace(
                            mat.as_str(),
                            &format!("\x1b[33;1m{}\x1b[0m", mat.as_str()),
                        );
                        println!("{}:{}: {}", file_path.display(), i + 1, highlighted);
                    }
                }
            }
        }
    });

    let locked = matches.lock().unwrap();
    let total = locked.len();

    // JSON
    if let Some(path) = json_out {
        let report = Report {
            pattern: regex_text.to_string(),
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
