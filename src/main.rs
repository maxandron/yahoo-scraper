use axum::extract::State;
use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::string::String;
use thirtyfour::WebDriver;

#[tokio::main]
async fn main() {
    println!("Starting...");

    // Load the .env file.
    dotenv().ok();

    println!("Initializing driver...");
    let driver = init_driver().await.unwrap();

    let app = Router::new()
        .route("/price/:ticker", get(scrape_price_handler))
        .with_state(driver);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on: http://127.0.0.1:3000");
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
