use qbit_rs::Qbit;
use qbit_rs::model::{Credential, AddTorrentArg, TorrentSource};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use reqwest::Client;
use reqwest::multipart;

#[tokio::main]
async fn main() {
    // Reemplaza "username" y "password" con tus credenciales válidas
    let credential = Credential::new("admin", "123456");
    // Reemplaza "http://localhost:8080" con la URL correcta de tu instancia de qBittorrent
    let api = Qbit::new("http://localhost:8080", credential);

    match api.get_version().await {
        Ok(version) => println!("qBittorrent version: {}", version),
        Err(e) => eprintln!("Failed to get version: {:?}", e),
    }

    // Ruta del archivo torrent en el escritorio
    let torrent_path = Path::new("C:/Users/emili/OneDrive/Escritorio/Ghost.of.Tsushima.Directors.Cut.elamigos.torrent");

     // Leer el archivo torrent
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

    // Verificar el tamaño del buffer leído
    println!("Buffer length: {}", buffer.len());

    // Crear el argumento para agregar el torrent
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

    // Crear cliente reqwest
    let client = Client::new();

    // Construir multipart/form-data
    let form = multipart::Form::new()
        .part("torrents", multipart::Part::bytes(buffer)
            .file_name("Ghost.of.Tsushima.Directors.Cut.elamigos.torrent"));

    // Intentar agregar el torrent y obtener la respuesta como texto
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
                } else {
                    eprintln!("Failed to add torrent. Status: {}", status);
                }
            },
            Err(e) => eprintln!("Failed to send request: {:?}", e),
    }
}