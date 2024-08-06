use clap::Parser;

mod simple_cube;
mod instanced_cube;
mod instanced_cube_dynamic_update;
mod utils;
mod test_glfw_adapter;

#[derive(Parser, Debug)]
#[command(name = "banshee_wgpu_adapter_test")]
#[command(version = "0.0.0")]
#[command(about = "Runs a specific test for the Banshee WGPU Adapter.")]
struct Args {
    #[arg(short, long)]
    test_name: String,
}


fn main() {
    structured_logger::Builder::new()
        .with_target_writer("*", structured_logger::json::new_writer(std::io::stdout()))
        .init();
    let output_folder = std::path::Path::new("test_outputs");
    if !output_folder.exists() {
        std::fs::create_dir(output_folder).unwrap();
    }
    let args = Args::parse();
    match args.test_name {
        test_name if test_name == "simple_cube" => {
            simple_cube::run(output_folder).unwrap();
        },
        test_name if test_name == "instanced_cube" => {
            instanced_cube::run(output_folder).unwrap();
        },
        test_name if test_name == "instanced_cube_dynamic_update" => {
            instanced_cube_dynamic_update::run(output_folder).unwrap();
        },
        test_name if test_name == "glfw_adapter" => {
            test_glfw_adapter::run().unwrap();
        },
        _ => {
            panic!("Unknown test name: {}", args.test_name);
        }
    }
}
