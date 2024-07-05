
    use json::parse;
    use qbit_rs::Qbit;
    use reqwest::Client;
    use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    // Definir variables estáticas mutables para las cookies y el cliente
    static mut COOKIES: Option<String> = None;
    static mut CLIENT: Option<Client> = None;
    static mut API_URL: Option<String> = None;


    pub async fn init(api_url: &str, username: &str, password: &str) {
        let client = Client::new();
        let cookies = match authenticate(&client, api_url, username, password).await {
            Ok(cookies) => cookies,
            Err(e) => {
                eprintln!("Failed to authenticate: {:?}", e);
                return;
            }
        };

        // Guardar el cliente, las cookies y el api_url en las variables estáticas mutables
        unsafe {
            COOKIES = Some(cookies);
            CLIENT = Some(client);
            API_URL = Some(api_url.to_string());
        }
    }

    pub async fn get_version(api: &Qbit) -> Result<String, Box<dyn std::error::Error>> {
        match api.get_version().await {
            Ok(version) => Ok(version),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn authenticate(client: &Client, api_url: &str, username: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
        let auth_url = format!("{}/api/v2/auth/login", api_url);
        let mut params = HashMap::new();
        params.insert("username", username);
        params.insert("password", password);

        let response = client.post(&auth_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to authenticate")));
        }

        let cookies = response
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .map(|header_value| header_value.to_str().unwrap())
            .collect::<Vec<&str>>()
            .join("; ");

        Ok(cookies)
    }

    pub async fn add_torrent(torrent_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Acceder de manera insegura a las variables globales
       // Acceder de manera insegura a las variables globales
       let (client, cookies, api_url) = unsafe {
        (
            CLIENT.as_ref().unwrap(),
            COOKIES.as_ref().unwrap(),
            API_URL.as_ref().unwrap(),
        )
    };

        let mut file = File::open(Path::new(torrent_path))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str(cookies).unwrap());
        headers.insert(REFERER, HeaderValue::from_str(api_url).unwrap());
        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        let form = reqwest::multipart::Form::new()
            .part("torrents", reqwest::multipart::Part::bytes(buffer).file_name("torrent"));

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

    pub async fn check_download_progress(torrent_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (client, cookies, api_url) = unsafe {
            (
                CLIENT.as_ref().unwrap(),
                COOKIES.as_ref().unwrap(),
                API_URL.as_ref().unwrap(),
            )
        };

        let url = format!("{}/api/v2/torrents/info", api_url);
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::COOKIE, HeaderValue::from_str(cookies).unwrap());

        let response = client.get(&url)
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            parse_torrents_info(&text, torrent_name);
        } else {
            eprintln!("Failed to fetch torrents. Status: {}", response.status());
        }

        Ok(())
    }
    
    fn parse_torrents_info(json_text: &str, torrent_name: &str) {
        let parsed = parse(json_text);
        match parsed {
            Ok(torrents) => {
                if torrents.is_array() {
                    for torrent in torrents.members() {
                        let name = torrent["name"].as_str().unwrap_or("Unknown");
                        if name == torrent_name {
                            let progress = torrent["progress"].as_f64().unwrap_or(0.0) * 100.0;
                            let state = torrent["state"].as_str().unwrap_or("Unknown");
                            let downloaded_size = torrent["downloaded"].as_f64().unwrap_or(0.0) / (1_024.0 * 1_024.0 * 1_024.0); // as GB
                            let download_speed = torrent["dlspeed"].as_f64().unwrap_or(0.0) / 1_000_000.0; // as MB/s
                            let total_size = torrent["total_size"].as_f64().unwrap_or(0.0) / (1_024.0 * 1_024.0 * 1_024.0); // as GB
                            let eta = torrent["eta"].as_i64().unwrap_or(0); // in seconds
                            let eta_minutes = eta / 60;
                            let eta_seconds = eta % 60;
                            
                            println!("--------------------------------");
                            println!("Estado: {}", state);
                            println!("Torrent: {} - Progreso: {:.2}%", name, progress);
                            println!("Velocidad de descarga: {:.2} MB/s", download_speed);
                            println!("Tamaño descargado: {:.2} GB", downloaded_size);
                            println!("Tamaño total: {:.2} GB", total_size);
                            println!("Tiempo estimado para terminar: {} minutos y {} segundos", eta_minutes, eta_seconds);
                            break; // Salir del bucle una vez que encontramos el torrent
                        }
                    }
                } else {
                    eprintln!("Expected an array in JSON response");
                }
            }
            Err(e) => eprintln!("Failed to parse JSON response: {:?}", e),
        }
    }
    