use tonic_prost_build::configure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Сообщить Cargo пересобирать при изменении proto/blog.proto
    println!("cargo:rerun-if-changed=../proto/blog.proto");

    // Компилируем proto файл
    configure()
        .compile_protos(&["../proto/blog.proto"], &["../proto"])
        .unwrap();

    Ok(())
}
