mod client;
pub mod parsing;
mod ui;

#[tokio::main]
async fn main() {
    parsing::run().await;
}
