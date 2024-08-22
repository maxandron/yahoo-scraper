use axum::extract::State;
use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::string::String;
use std::{error::Error, process::Stdio};
use thirtyfour::WebDriver;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    println!("Starting...");

    // Load the .env file.
    dotenv().ok();

    //// Kill any existing geckodriver processes
    //println!("Killing existing geckodriver processes if any are running");
    //let _ = tokio::process::Command::new("pkill")
    //    .arg("geckodriver")
    //    .output()
    //    .await;
    //
    //let geckodriver_path = std::env::var("GECKODRIVER_PATH").unwrap_or("geckodriver".to_string());
    //println!("Running geckodriver ({geckodriver_path})");
    //let child = tokio::process::Command::new(geckodriver_path)
    //    .stdout(Stdio::piped())
    //    .spawn()
    //    .expect("Failed to start geckodriver");
    //
    //println!("Waiting for geckodriver to start...");
    //let stdout = child.stdout.expect("Failed to get stdout");
    //
    //// Combine stdout and stderr into a single stream.
    //let mut reader = BufReader::new(stdout).lines();
    //
    //while let Some(line) = reader.next_line().await.unwrap() {
    //    println!("Received: {line}");
    //    if line.contains("Listening") {
    //        break;
    //    }
    //}
    //println!("Geckodriver started");

    println!("Initializing driver...");
    let driver = init_driver().await.unwrap();

    let app = Router::new()
        .route("/price/:ticker", get(scrape_price_handler))
        .with_state(driver);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct Params {
    ticker: String,
}

async fn scrape_price_handler(
    Path(params): Path<Params>,
    State(driver): State<WebDriver>,
) -> (StatusCode, Json<String>) {
    let price = match scrape_price(&driver, &params.ticker).await {
        Ok(price) => price,
        Err(e) => {
            eprintln!("Error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal Server Error".to_string()),
            );
        }
    };
    (StatusCode::OK, Json(price))
}

async fn init_driver() -> Result<WebDriver, Box<dyn Error + Send + Sync>> {
    let mut caps = thirtyfour::DesiredCapabilities::firefox();
    caps.set_headless()?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    let driver = thirtyfour::WebDriver::new("http://127.0.0.1:4444", caps).await?;

    Ok(driver)
}

async fn scrape_price(
    driver: &thirtyfour::WebDriver,
    ticker: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!("https://finance.yahoo.com/quote/{ticker}", ticker = ticker);
    driver.goto(url).await?;

    let price_streamer = driver.find(thirtyfour::By::ClassName("livePrice")).await?;
    let price_span = price_streamer.find(thirtyfour::By::Tag("span")).await?;
    let price = price_span.inner_html().await?;

    Ok(price)
}
