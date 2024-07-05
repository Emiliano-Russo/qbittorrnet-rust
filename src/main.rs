mod index;

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let api_url = "http://localhost:8080";
    let username = "admin";
    let password = "123456";

    index::init(api_url, username, password).await;

    let torrent_path = "C:/Users/emili/OneDrive/Escritorio/Ghost.of.Tsushima.Directors.Cut.elamigos.torrent";
    match index::add_torrent(torrent_path).await {
        Ok(_) => println!("Torrent added successfully"),
        Err(e) => eprintln!("Failed to add torrent: {:?}", e),
    }
    
    loop {
        match index::check_download_progress("Ghost.of.Tsushima.Directors.Cut.elamigos").await {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to check download progress: {:?}", e),
        }
        sleep(Duration::from_secs(10)).await;
    }
}
