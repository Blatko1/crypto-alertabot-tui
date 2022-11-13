mod bot;
mod console;
mod error;
mod input;
mod save;

fn main() {
    let save_data = match save::load_create_save_file() {
        Ok(data) => data,
        Err(err) => panic!("Alertabot Error: {}", err),
    };
}
