mod catchers;
mod cors;
mod macros;

use crate::{
    error::Error::{self, ConfigurationError},
    http::catchers::{
        bad_request, forbidden, internal_server_error, not_authorized, not_found,
        too_many_requests, unprocessed_entity,
    },
    utils::generate_cert,
    Result,
};
use log::info;
use rocket::{
    data::{Limits, ToByteUnit},
    {post, routes, Config, Data},
};
use serde::{de, Deserialize, Deserializer};
use std::{fs::File, io::Read, net::IpAddr, path::Path};

const BUFFER_SIZE: usize = 4096;

const SRV_ADDR: &str = "127.0.0.1";
const SRV_PORT: usize = 8080;
const SRV_KEEP_ALIVE: usize = 60;
const SRV_FORMS_LIMIT: usize = 1024 * 256;
const SRV_JSON_LIMIT: usize = 1024 * 256;
const SRV_SECRET_KEY: &str = "t/xZkYvxfC8CSfTSH9ANiIR9t1SvLHqOYZ7vH4fp11s=";
const SSL_ENABLED: bool = false;
const SSL_GENERATE_SELF_SIGNED: bool = true;
const SSL_KEY_FILE: &str = "./private/key";
const SSL_CERT_FILE: &str = "./private/cert";

/// Rocket API Server parameters
#[derive(Deserialize, Clone, Debug, Default)]
pub struct HttpServerSettings {
    /// Server config related parameters
    #[serde(default)]
    pub server: ServerConfig,

    /// SSL Configuration
    #[serde(default, deserialize_with = "configure_ssl")]
    pub ssl: Option<SslConfig>,
}

impl HttpServerSettings {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        //! Read configuration settings from yaml file
        //!
        //! ## Example usage
        //! ```ignore
        //! HttpServerSettings::from_file("config.sample.yml");
        //! ```
        //!
        match config::Config::builder()
            .add_source(config::File::with_name(path.as_ref().to_str().unwrap()))
            .build()
        {
            Ok(c) => match c.try_deserialize() {
                Ok(cfg) => Ok(cfg),
                Err(e) => Err(ConfigurationError(e.to_string())),
            },
            Err(e) => Err(ConfigurationError(e.to_string())),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        info!("Initializing settings..");
        let ssl_config = self.ssl.clone();
        if let Some(mut s) = ssl_config {
            if s.enabled && s.generate_self_signed {
                // SSL is enabled, and generate self signed certificate is enabled
                let certs = generate_cert();
                s.pem_certificate = Some(certs.x509_certificate.to_pem().unwrap());
                s.pem_private_key = Some(certs.private_key.private_key_to_pem_pkcs8().unwrap());
            } else if s.enabled && !s.generate_self_signed {
                // SSL is enabled, and generate self signed certificate is disabled
                if s.key_file.is_empty() || s.cert_file.is_empty() {
                    return Err(ConfigurationError(
                        "key_file and/or cert_file is empty".to_string(),
                    ));
                } else if !Path::new(&s.key_file).is_file() || !Path::new(&s.cert_file).is_file() {
                    return Err(ConfigurationError(
                        "key_file and/or cert_file not available".to_string(),
                    ));
                } else {
                    // read key
                    let mut key = Vec::new();
                    {
                        let mut kf = File::open(&s.key_file).unwrap();
                        kf.read_to_end(&mut key).unwrap();
                    }
                    // read certificate
                    let mut cert = Vec::new();
                    {
                        let mut cf = File::open(&s.cert_file).unwrap();
                        cf.read_to_end(&mut cert).unwrap();
                    }
                    s.pem_certificate = Some(cert);
                    s.pem_private_key = Some(key);
                }
            }
            self.ssl = Some(s);
        }
        Ok(())
    }
}

/// Rocket Server params
#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    /// Server Ip Address to start Rocket API Server
    #[serde(default = "default_server_host")]
    pub host: IpAddr,
    /// Server port to listen Rocket API Server
    #[serde(default = "default_server_port")]
    pub port: usize,
    /// Server Keep Alive
    #[serde(default = "default_server_keep_alive")]
    pub keep_alive: usize,
    /// Forms limitation
    #[serde(default = "default_server_forms_limit")]
    pub forms_limit: usize,
    /// JSON transfer limitation
    #[serde(default = "default_server_json_limit")]
    pub json_limit: usize,
    /// Api Server Secret key
    #[serde(default = "default_server_secret_key")]
    pub secret_key: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: SRV_ADDR.parse().unwrap(),
            port: SRV_PORT,
            keep_alive: SRV_KEEP_ALIVE,
            forms_limit: SRV_FORMS_LIMIT,
            json_limit: SRV_JSON_LIMIT,
            secret_key: SRV_SECRET_KEY.into(),
        }
    }
}

/// Server SSL params
#[derive(Deserialize, Clone, Debug)]
pub struct SslConfig {
    /// Enabled: yes/no
    #[serde(default = "default_ssl_enabled")]
    pub enabled: bool,
    /// Let the server generate a self-signed pair: yes/no
    #[serde(default = "default_ssl_self_signed")]
    pub generate_self_signed: bool,
    /// key file (if generate_self_signed is `NO`)
    #[serde(default = "default_ssl_key_file")]
    pub key_file: String,
    /// certificate pem file (if generate_self_signed is `NO`)
    #[serde(default = "default_ssl_cert_file")]
    pub cert_file: String,

    // Not to be included in config file
    // hidden and for use with rocket app
    pub pem_certificate: Option<Vec<u8>>,
    pub pem_private_key: Option<Vec<u8>>,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: SSL_ENABLED,
            generate_self_signed: SSL_GENERATE_SELF_SIGNED,
            key_file: SSL_KEY_FILE.into(),
            cert_file: SSL_CERT_FILE.into(),
            pem_certificate: None,
            pem_private_key: None,
        }
    }
}

// All Server defaults
fn default_server_host() -> IpAddr {
    SRV_ADDR.parse().unwrap()
}

fn default_server_port() -> usize {
    SRV_PORT
}

fn default_server_keep_alive() -> usize {
    SRV_KEEP_ALIVE
}

fn default_server_forms_limit() -> usize {
    SRV_FORMS_LIMIT
}

fn default_server_json_limit() -> usize {
    SRV_JSON_LIMIT
}

fn default_server_secret_key() -> String {
    SRV_SECRET_KEY.into()
}

// All SSL config defaults
fn default_ssl_enabled() -> bool {
    SSL_ENABLED
}

fn default_ssl_self_signed() -> bool {
    SSL_GENERATE_SELF_SIGNED
}

fn default_ssl_key_file() -> String {
    SSL_KEY_FILE.into()
}

fn default_ssl_cert_file() -> String {
    SSL_CERT_FILE.into()
}

/// SSL configuration deserializer
fn configure_ssl<'de, D>(deserializer: D) -> std::result::Result<Option<SslConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let ssl_config: Option<SslConfig> = Option::deserialize(deserializer)?;
    match ssl_config {
        Some(mut s) => {
            if s.enabled && s.generate_self_signed {
                // SSL is enabled, and generate self signed certificate is enabled
                let certs = generate_cert();
                s.pem_certificate = Some(certs.x509_certificate.to_pem().unwrap());
                s.pem_private_key = Some(certs.private_key.private_key_to_pem_pkcs8().unwrap());
            } else if s.enabled && !s.generate_self_signed {
                // SSL is enabled, and generate self signed certificate is disabled
                if s.key_file.is_empty() || s.cert_file.is_empty() {
                    return Err(de::Error::custom("key_file and/or cert_file is empty"));
                } else if !Path::new(&s.key_file).is_file() || !Path::new(&s.cert_file).is_file() {
                    return Err(de::Error::custom("key_file and/or cert_file not available"));
                } else {
                    // read key
                    let mut key = Vec::new();
                    {
                        let mut kf = File::open(&s.key_file).unwrap();
                        kf.read_to_end(&mut key).unwrap();
                    }
                    // read certificate
                    let mut cert = Vec::new();
                    {
                        let mut cf = File::open(&s.cert_file).unwrap();
                        cf.read_to_end(&mut cert).unwrap();
                    }
                    s.pem_certificate = Some(cert);
                    s.pem_private_key = Some(key);
                }
            }
            Ok(Some(s))
        }
        None => Ok(None),
    }
}

#[post("/", data = "<data>")]
pub async fn handle_connection(data: Data<'_>) {
    // Stream the data to stdout for now.
    let _bytes_count = data
        .open(BUFFER_SIZE.kibibytes())
        .stream_to(tokio::io::stdout())
        .await
        .unwrap();
}

pub async fn create_http_server(settings: HttpServerSettings) -> Result<()> {
    let server_settings = settings.server;

    let limits = Limits::new()
        .limit("forms", server_settings.forms_limit.into())
        .limit("json", server_settings.json_limit.into());

    let rocket_cfg = Config::figment()
        .merge(("address", server_settings.host.to_string()))
        .merge(("port", server_settings.port as u16))
        .merge(("limits", limits))
        .merge(("secret_key", server_settings.secret_key.as_str()))
        .merge(("keep_alive", server_settings.keep_alive as u32));

    // Configure SSL status for the api server
    let rocket_cfg = if let Some(ssl_cfg) = settings.ssl {
        if ssl_cfg.enabled {
            // ssl is enabled
            if ssl_cfg.pem_certificate.is_some() && ssl_cfg.pem_private_key.is_some() {
                // merge the certs & key into rocket config
                rocket_cfg
                    .merge(("tls.certs", ssl_cfg.pem_certificate))
                    .merge(("tls.key", ssl_cfg.pem_private_key))
            } else {
                // ssl certificate info not available
                return Err(Error::SslCertificateError);
            }
        } else {
            // ssl not enabled
            rocket_cfg
        }
    } else {
        // no ssl configuration
        rocket_cfg
    };

    // Configure the Rocket server with configured settings
    let app = rocket::custom(rocket_cfg).attach(cors::Cors);

    // Catchers
    let app = app.register(
        "/",
        rocket::catchers![
            bad_request,
            forbidden,
            not_authorized,
            not_found,
            unprocessed_entity,
            internal_server_error,
            too_many_requests,
        ],
    );

    // Add the index route
    let app = app.mount("/", routes![handle_connection]);
    let _ = app.launch().await;
    Ok(())
}
