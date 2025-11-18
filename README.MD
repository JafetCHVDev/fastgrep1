# fastgrep


CLI multihilo para buscar patrones en repositorios grandes.


## Requisitos
- Rust toolchain (rustup + cargo)
- Docker (opcional)


## Compilar y ejecutar localmente


```bash
cargo build --release
./target/release/fastgrep "fn main" ./src -t 4