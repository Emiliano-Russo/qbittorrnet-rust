
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process::Command;


    #[cfg(windows)]
    fn is_executable(path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "exe")
    }

    // Función para buscar todos los archivos ejecutables en un directorio
    pub fn find_executables(dir: &str) -> io::Result<Vec<PathBuf>> {
        let path = Path::new(dir);
        let mut executables = Vec::new();

        if !path.is_dir() {
            return Ok(executables);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && is_executable(&path) {
                executables.push(path);
            }
        }

        Ok(executables)
    }

    // Función para ejecutar el archivo encontrado
    pub fn run_executable(executable_path: &Path) -> io::Result<()> {
        let mut command = Command::new(executable_path);
        command.status()?;
        Ok(())
    }

    // Función para ejecutar un archivo con permisos elevados en Windows
    pub fn run_executable_with_elevation(executable_path: &Path) -> io::Result<()> {
        Command::new("powershell")
            .arg("-Command")
            .arg("Start-Process")
            .arg("-FilePath")
            .arg(format!("'{}'", executable_path.display())) // Asegurarse de que el argumento esté entrecomillado
            .arg("-Verb")
            .arg("runAs")
            .spawn()?
            .wait()?;
        Ok(())
    }

