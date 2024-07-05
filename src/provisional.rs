mod api;
mod auth;
mod torrent;

use api::get_version;
use auth::{authenticate, get_cookies};
use torrent::{add_torrent, check_download_progress};
use reqwest::Client;
use qbit_rs::Qbit;
use qbit_rs::model::Credential;

#[tokio::main]
async fn main() {
    let username = "admin";
    let password = "123456";
    let api_url = "http://localhost:8080";

    let api = Qbit::new(api_url, Credential::new(username, password));

    match get_version(&api).await {
        Ok(version) => println!("qBittorrent version: {}", version),
        Err(e) => eprintln!("Failed to get version: {:?}", e),
    }

    let client = Client::new();
    let cookies = match authenticate(&client, api_url, username, password).await {
        Ok(cookies) => cookies,
        Err(e) => {
            eprintln!("Failed to authenticate: {:?}", e);
            return;
        }
    };

    let torrent_path = "C:/Users/emili/OneDrive/Escritorio/Ghost.of.Tsushima.Directors.Cut.elamigos.torrent";
    match add_torrent(&client, api_url, &cookies, torrent_path).await {
        Ok(_) => println!("Torrent added successfully"),
        Err(e) => eprintln!("Failed to add torrent: {:?}", e),
    }

    check_download_progress(&client, &cookies, api_url).await;
}
