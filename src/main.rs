mod search;
mod types;


use clap::Parser;
use search::search_pattern;


#[derive(Parser)]
#[command(author, version, about = "fastgrep — búsqueda rápida multihilo")]
struct Args {
/// Patrón regex a buscar
pattern: String,


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
}


fn main() {
let args = Args::parse();


if let Err(e) = search_pattern(
&args.pattern,
&args.path,
args.list,
args.threads,
args.json.as_deref(),
) {
eprintln!("Error: {}", e);
}
}