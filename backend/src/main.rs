mod handler;

#[tokio::main]
async fn main() {
    let app = crate::handler::create_route();

    poem::Server::new(poem::listener::TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
        .expect("failed to serve HTTP");
}
