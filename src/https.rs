use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio_rustls::TlsAcceptor;

pub fn load_tls_config(cert_path: &str, key_path: &str) -> Result<Arc<ServerConfig>, Box<dyn std::error::Error>> {
    // Load certificate chain
    let cert_file = File::open(cert_path)?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();

    // Load private key
    let key_file = File::open(key_path)?;
    let mut key_reader = BufReader::new(key_file);
    let mut keys = pkcs8_private_keys(&mut key_reader)?;
    
    if keys.is_empty() {
        return Err("No PKCS8 private keys found".into());
    }
    
    let private_key = PrivateKey(keys.remove(0));

    // Create TLS config
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;

    Ok(Arc::new(config))
}

pub fn create_tls_acceptor(config: Arc<ServerConfig>) -> TlsAcceptor {
    TlsAcceptor::from(config)
}

pub fn generate_self_signed_cert() -> Result<(), Box<dyn std::error::Error>> {
    // This is a placeholder for self-signed certificate generation
    // In production, you should use proper certificates from Let's Encrypt or a CA
    println!("To use HTTPS, provide certificate files:");
    println!("  - Certificate: cert.pem");
    println!("  - Private Key: key.pem");
    println!("Or set CERT_PATH and KEY_PATH environment variables.");
    println!();
    println!("For development, you can generate self-signed certificates with:");
    println!("  openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes");
    
    Ok(())
}