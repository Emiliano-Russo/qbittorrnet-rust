use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, REFERER};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use json::parse;

pub async fn add_torrent(client: &Client, api_url: &str, cookies: &str, torrent_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(Path::new(torrent_path))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, HeaderValue::from_str(cookies).unwrap());
    headers.insert(REFERER, HeaderValue::from_str(api_url).unwrap());
    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let form = reqwest::multipart::Form::new()
        .part("torrents", reqwest::multipart::Part::bytes(buffer)
            .file_name("torrent"));

    let add_torrent_url = format!("{}/api/v2/torrents/add", api_url);
    let response = client.post(&add_torrent_url)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to add torrent")))
    }
}

pub async fn check_download_progress(client: &Client, cookies: &str, api_url: &str) {
    let url = format!("{}/api/v2/torrents/info", api_url);
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, HeaderValue::from_str(cookies).unwrap());

    loop {
        match client.get(&url)
            .headers(headers.clone())
            .send()
            .await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(text) => {
                            parse_torrents_info(&text);
                        }
                        Err(e) => eprintln!("Failed to get response text: {:?}", e),
                    }
                } else {
                    eprintln!("Failed to fetch torrents. Status: {}", response.status());
                }
            }
            Err(e) => eprintln!("Failed to send request: {:?}", e),
        }
        sleep(Duration::from_secs(10)).await;
    }
}

fn parse_torrents_info(json_text: &str) {
    let parsed = parse(json_text);
    match parsed {
        Ok(torrents) => {
            if torrents.is_array() {
                for torrent in torrents.members() {
                    let name = torrent["name"].as_str().unwrap_or("Unknown");
                    let progress = torrent["progress"].as_f64().unwrap_or(0.0) * 100.0;
                    let download_speed = torrent["dlspeed"].as_f64().unwrap_or(0.0) / 1_000_000.0; // as MB/s
                    let total_size = torrent["total_size"].as_f64().unwrap_or(0.0) / (1_024.0 * 1_024.0 * 1_024.0); // as GB
                    let eta = torrent["eta"].as_i64().unwrap_or(0); // in seconds
                    let eta_minutes = eta / 60;
                    let eta_seconds = eta % 60;

                    println!("Torrent: {} - Progreso: {:.2}%", name, progress);
                    println!("Velocidad de descarga: {:.2} MB/s", download_speed);
                    println!("Tamaño total: {:.2} GB", total_size);
                    println!("Tiempo estimado para terminar: {} minutos y {} segundos", eta_minutes, eta_seconds);
                }
            } else {
                eprintln!("Expected an array in JSON response");
            }
        }
        Err(e) => eprintln!("Failed to parse JSON response: {:?}", e),
    }
}
