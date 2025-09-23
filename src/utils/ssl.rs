use crate::utils::database::project_data_dir;
use std::path::PathBuf;
use rcgen::generate_simple_self_signed;

/// Generate a self-signed certificate and key with the given subjectAltNames,
/// writing them to the given paths.
fn generate_certificates(
    cert_path: &PathBuf,
    key_path: &PathBuf,
    subject_alt_names: Vec<String>,
) -> std::io::Result<()> {
    let cert_key = generate_simple_self_signed(subject_alt_names)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let cert_pem = cert_key.cert.pem();
    let key_pem = cert_key.signing_key.serialize_pem();

    std::fs::write(cert_path, cert_pem)?;
    std::fs::write(key_path, key_pem)?;

    Ok(())
}

/// Ensures a snakeoil cert and key exist in the project data dir.
/// Returns (cert_path, key_path).
pub fn ensure_dev_certificates() -> std::io::Result<(PathBuf, PathBuf)> {
    let data_dir = project_data_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "No project data dir available")
    })?;

    std::fs::create_dir_all(&data_dir)?;

    let cert_path = data_dir.join("dev_cert.pem");
    let key_path = data_dir.join("dev_key.pem");

    if !cert_path.exists() || !key_path.exists() {
        println!("Generating self-signed development certificate...");
        generate_certificates(&cert_path, &key_path, vec!["localhost".to_string()])?;
    }

    Ok((cert_path, key_path))
}

/// Returns the paths to either real cert/key (if present) or the dev cert/key (ensured).
pub fn get_certificates() -> std::io::Result<(PathBuf, PathBuf)> {
    let le_cert = PathBuf::from("/etc/letsencrypt/live/media.aiowa.com/fullchain.pem");
    let le_key = PathBuf::from("/etc/letsencrypt/live/media.aiowa.com/privkey.pem");
    if le_cert.exists() && le_key.exists() {
        return Ok((le_cert, le_key));
    }
    

    if let Ok(cert_path_env) = std::env::var("CERT_PATH") {
        let base = PathBuf::from(cert_path_env);
        let candidates = [
            (base.join("fullchain.pem"), base.join("privkey.pem")),
            (base.join("cert.pem"), base.join("key.pem")),
        ];
        for (cert, key) in candidates {
            if cert.exists() && key.exists() {
                return Ok((cert, key));
            }
        }
    }

    let data_dir = project_data_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "No project data dir available")
    })?;

    let real_cert = data_dir.join("cert.pem");
    let real_key = data_dir.join("key.pem");

    if real_cert.exists() && real_key.exists() {
        Ok((real_cert, real_key))
    } else {
        ensure_dev_certificates()
    }
}
