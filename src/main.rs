mod search;
mod types;

use clap::{Parser, ArgAction, ValueEnum};
use colored::*;
use atty; // NECESARIO
use search::{search_pattern, smart_pattern};
use types::{Report};

#[derive(ValueEnum, Clone, Debug)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(ValueEnum, Clone, Debug)]
enum SortMode {
    File,
    Line,
    Match,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Fastgrep PRO — búsqueda rápida multihilo con tabla, colores y patrones inteligentes"
)]
struct Args {
    /// Patrón regex a buscar (o usar --smart)
    pattern: Option<String>,

    /// Directorio donde buscar
    path: String,

    /// Mostrar solo archivos que contienen coincidencias
    #[arg(short, long)]
    list: bool,

    /// Número de hilos para búsqueda (por defecto = núcleos)
    #[arg(short = 't', long, default_value_t = num_cpus::get())]
    threads: usize,

    /// Guardar resultado en JSON (ruta de archivo)
    #[arg(long)]
    json: Option<String>,

    /// Mostrar resultados en formato tabla PRO
    #[arg(long, action = ArgAction::SetTrue)]
    table: bool,

    /// Usar patrón inteligente: email, ipv4, url, jwt, token, uuid, creditcard, hex
    #[arg(long)]
    smart: Option<String>,

    /// Modo de color: auto, always, never
    #[arg(long, default_value = "auto")]
    color: ColorMode,

    /// Ordenar resultados: file, line, match
    #[arg(long)]
    sort: Option<SortMode>,
}

/// Control del uso de colores
fn colorize(enable: bool, text: &str, style: fn(&str) -> ColoredString) -> String {
    if enable {
        style(text).to_string()
    } else {
        text.to_string()
    }
}

/// Renderizado de tabla PRO en terminal
fn print_table(report: &Report, use_colors: bool) {
    println!("{}", colorize(use_colors, "FASTGREP — RESULTADOS", |t| t.bold().blue()));

    println!("┌──────────────────────────────────────────────┬────────┬────────────────────────────────────────────┐");
    println!("│ FILE                                         │ LINE   │ MATCH                                      │");
    println!("├──────────────────────────────────────────────┼────────┼────────────────────────────────────────────┤");

    for m in &report.matches {

        let highlighted = if use_colors {
            format!(
                "{}\x1b[33;1m{}\x1b[0m{}",
                "", 
                m.matched_text, 
                ""
            )
        } else {
            m.matched_text.clone()
        };

        println!(
            "│ {:<44} │ {:>6} │ {} │",
            if use_colors { m.file.green().to_string() } else { m.file.clone() },
            m.line,
            highlighted
        );
    }

    println!("└──────────────────────────────────────────────┴────────┴────────────────────────────────────────────┘");
}

fn main() {
    let args = Args::parse();

    // Determinar si se usan colores
    let use_colors = match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    // Selección del patrón (normal o inteligente)
    let final_pattern = if let Some(kind) = args.smart.as_deref() {
        smart_pattern(kind).expect("Tipo de patrón inteligente no válido (email, ipv4, url...)")
    } else {
        args.pattern.clone().expect("Debes indicar un patrón o usar --smart <tipo>")
    };

    // Ejecutar búsqueda
    if let Err(e) = search_pattern(
        &final_pattern,
        &args.path,
        args.list,
        args.threads,
        args.json.as_deref(),
    ) {
        eprintln!("Error: {}", e);
        return;
    }

    // Si no se pidió tabla, terminamos aquí
    if !args.table {
        return;
    }

    // Tabla requiere JSON
    if args.json.is_none() {
        eprintln!("{}", colorize(use_colors, "Advertencia: --table requiere --json para construir tabla.", |t| t.red().bold()));
        return;
    }

    // Cargar reporte JSON generado internamente
    let json_path = args.json.as_ref().unwrap();
    let content = std::fs::read_to_string(json_path).expect("No se pudo leer JSON");
    let mut report: Report = serde_json::from_str(&content).expect("JSON inválido");

    // ORDENAMIENTO
    if let Some(mode) = args.sort {
        match mode {
            SortMode::File => report.matches.sort_by(|a, b| a.file.cmp(&b.file)),
            SortMode::Line => report.matches.sort_by(|a, b| a.line.cmp(&b.line)),
            SortMode::Match => report.matches.sort_by(|a, b| a.matched_text.cmp(&b.matched_text)),
        }
    }

    // Mostrar tabla PRO
    print_table(&report, use_colors);
}
