// use std::io;
// use std::io::Write;

use tokio::sync::mpsc;

use crate::agent::agent::Agent;

// const SMALL_MODEL: &str = "qwen3:0.6b";
const SMALL_MODEL: &str = "phi3.5";

#[tokio::main]
async fn main() {
    let (tx1, rx1) = mpsc::channel(10);
    let (tx2, rx2) = mpsc::channel(10);

    let _ = tx1.send("Materialism is the best way to live. I'd rather shop than to think about anything else".into()).await;
    let ag1 = Agent::new(
        "llama",
        SMALL_MODEL,
        "You are a debating participant named llama. Your sole purpose and objective is to argue and convince that materialism only leads to suffering. Keeps the points very short and quick.",
        rx1,
        tx2,
    );

    let ag2 = Agent::new(
        "ggoat",
        SMALL_MODEL,
        "You are a debating participant named ggoat. Your sole purpose and objective is to argue and convince that materialism is the BEST way to live life. Keeps the points very short and quick. DO NOT BACK DOWN AND ONLY FOCUS ON ARGUING THAT MATERIALISM IS THE BEST WAY TO LIVE",
        rx2,
        tx1,
    );

    let ag1_task = tokio::spawn(ag1.run());
    let ag2_task = tokio::spawn(ag2.run());

    let _ = tokio::join!(ag1_task, ag2_task);
}

pub mod agent;
