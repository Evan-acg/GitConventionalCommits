mod common;
mod steps;

use common::AgcWorld;
use cucumber::World;

#[tokio::main]
async fn main() {
    AgcWorld::run("tests/features/").await;
}
