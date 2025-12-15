use tonic::include_proto;

// Включаем сгенерированный код
include_proto!("blog");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = RegisterRequest {
        username: "user123".to_string(),
        email: "user@example.com".to_string(),
        password: "securepassword".to_string(),
    };

    println!("Created register request for user: {}", request.username);

    Ok(())
}
