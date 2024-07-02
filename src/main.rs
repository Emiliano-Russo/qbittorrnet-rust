use qbit_rs::Qbit;
use qbit_rs::model::{Credential, AddTorrentArg, TorrentSource};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER};
use std::time::Duration;
use tokio::time::sleep;
use json::parse;

#[tokio::main]
async fn main() {
    let username = "admin";
    let password = "123456";

    let api = Qbit::new("http://localhost:8080", Credential::new(username, password));

    match api.get_version().await {
        Ok(version) => println!("qBittorrent version: {}", version),
        Err(e) => eprintln!("Failed to get version: {:?}", e),
    }

    let client = Client::new();
    let auth_url = "http://localhost:8080/api/v2/auth/login";
    let params = [("username", username), ("password", password)];

    let response = client.post(auth_url)
        .form(&params)
        .send()
        .await
        .expect("Failed to send auth request");

    if !response.status().is_success() {
        eprintln!("Failed to authenticate. Status: {}", response.status());
        return;
    }

    let cookies = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|header_value| header_value.to_str().unwrap())
        .collect::<Vec<&str>>()
        .join("; ");

    let torrent_path = Path::new("C:/Users/emili/OneDrive/Escritorio/Ghost.of.Tsushima.Directors.Cut.elamigos.torrent");

    let mut file = match File::open(&torrent_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open torrent file: {:?}", e);
            return;
        }
    };

    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        eprintln!("Failed to read torrent file: {:?}", e);
        return;
    }

    println!("Buffer length: {}", buffer.len());

    let add_torrent_arg = AddTorrentArg {
        source: TorrentSource::TorrentFiles { torrents: buffer.clone() },
        savepath: None,
        cookie: None,
        category: None,
        tags: None,
        skip_checking: None,
        paused: None,
        root_folder: None,
        rename: None,
        up_limit: None,
        download_limit: None,
        ratio_limit: None,
        seeding_time_limit: None,
        auto_torrent_management: None,
        sequential_download: None,
        first_last_piece_priority: None,
    };

    println!("Creating new reqwest client");

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookies).unwrap());
    headers.insert(REFERER, HeaderValue::from_str("http://localhost:8080").unwrap());
    let client = Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let form = reqwest::multipart::Form::new()
        .part("torrents", reqwest::multipart::Part::bytes(buffer)
            .file_name("Ghost.of.Tsushima.Directors.Cut.elamigos.torrent"));

    match client.post("http://localhost:8080/api/v2/torrents/add")
        .multipart(form)
        .send()
        .await {
            Ok(mut response) => {
                let status = response.status();
                let response_body = response.text().await.unwrap_or_else(|_| "Failed to get response text".to_string());
                println!("Response Body: {}", response_body);

                if status.is_success() {
                    println!("Torrent added successfully");
                    check_download_progress(&client, &cookies).await;
                } else {
                    eprintln!("Failed to add torrent. Status: {}", status);
                }
            },
            Err(e) => eprintln!("Failed to send request: {:?}", e),
    }
}

async fn check_download_progress(client: &Client, cookies: &str) {
    let url = "http://localhost:8080/api/v2/torrents/info";
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(cookies).unwrap());

    loop {
        match client.get(url)
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
