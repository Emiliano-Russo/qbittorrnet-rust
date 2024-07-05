mod index;
mod installation;

#[tokio::main]
async fn main() {
    let api_url = "http://localhost:8080";
    let username = "admin";
    let password = "123456";

    let dir = "C:/Users/emili/Downloads/The Genesis Order [FitGirl Repack]";
    match installation::find_executables(dir) {
        Ok(executables) => {
            if executables.is_empty() {
                println!("No hay archivos ejecutables en el directorio.");
            } else {
                println!("Archivos ejecutables disponibles en el directorio:");
                for executable in &executables {
                    if let Some(name) = executable.file_name() {
                        println!("{}", name.to_string_lossy());
                    }
                }
                
                // Ejecutar el primer ejecutable encontrado
                if let Some(first_executable) = executables.first() {
                    match installation::run_executable_with_elevation(first_executable) {
                        Ok(_) => println!("Ejecutable ejecutado con éxito."),
                        Err(err) => eprintln!("Error al ejecutar el archivo: {}", err),
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error al buscar archivos ejecutables: {}", err);
        }
    }
    


    // index::init(api_url, username, password).await;

    // let torrent_path = "C:/Users/emili/OneDrive/Escritorio/Ghost.of.Tsushima.Directors.Cut.elamigos.torrent";
    // match index::add_torrent(torrent_path).await {
    //     Ok(_) => println!("Torrent added successfully"),
    //     Err(e) => eprintln!("Failed to add torrent: {:?}", e),
    // }
    
    // loop {
    //     match index::check_download_progress("Ghost.of.Tsushima.Directors.Cut.elamigos").await {
    //         Ok(_) => (),
    //         Err(e) => eprintln!("Failed to check download progress: {:?}", e),
    //     }
    //     sleep(Duration::from_secs(10)).await;
    // }
}
