//! Code and utils for Initializing the connection to the server.
use std::sync::Arc;

use common_net::msg::ServerInfo;
use network::{ConnectAddr, Network};
use rustls::client::danger::ServerCertVerified;
use tracing::warn;

use crate::{Error, addr};

#[derive(Clone)]
pub enum Credentials {
    UsernamePassword(String, String),
    Username(String),
}

pub trait CredentialProvider {
    ///provides a quick check to verify if a Auth Server is expected and
    /// trusted This function is expected to return quickly.
    fn auth_trusted(&self, auth_provider: &Option<String>) -> bool;

    ///Called by the client once we are connected to the server and the server
    /// is requiring us to provide Credentials.
    /// This function is expected to trigger UI to wait for a result. So the
    /// User might take a minute or so to log in.
    /// Throw Error in case you want to abort login.
    fn get_credentials(
        &self,
        auth_provider: &Option<String>,
    ) -> impl std::future::Future<Output = Result<Credentials, ()>> + Send;
}

/// Use in case you already know your Auth provider Username and Password
/// upfront.
#[derive(Default)]
pub struct StaticCredentialProvider {
    pub credentials_by_auth: std::collections::HashMap<Option<String>, Credentials>,
}

impl CredentialProvider for StaticCredentialProvider {
    fn auth_trusted(&self, auth_provider: &Option<String>) -> bool {
        self.credentials_by_auth.contains_key(auth_provider)
    }

    async fn get_credentials(&self, auth_provider: &Option<String>) -> Result<Credentials, ()> {
        self.credentials_by_auth
            .get(auth_provider)
            .cloned()
            .ok_or(())
    }
}

#[derive(Debug)]
pub enum ClientInitStage {
    /// A connection to the server is being created
    ConnectionEstablish,
    /// Waiting for server version
    WatingForServerVersion,
    /// Server Version successfuly arrived, do a first PING
    ServerVersionArrived(ServerInfo),
    /// We're currently authenticating with the server
    Authentication,
    /// Loading map data, site information, recipe information and other
    /// initialization data
    LoadingInitData,
    /// Prepare data received by the server to be used by the client (insert
    /// data into the ECS, render map)
    StartingClient,
    /// Requesting Plugins from Server and downloading them
    RequestingPlugins,
}

/// Use this to get the "token_or_username" which is required by a server, token
/// if auth is used, username if it runs without auth.
/// NOTE: only call it if you trust the auth_provider!
pub(crate) async fn resolve_token_or_username(
    credentials: Credentials,
    trusted_auth_provider: &Option<String>,
) -> Result<String, Error> {
    match credentials {
        Credentials::Username(username) => Ok(username),
        Credentials::UsernamePassword(username, password) => match &trusted_auth_provider {
            Some(addr) => {
                let (scheme, authority) = match addr.split_once("://") {
                    Some((s, a)) => (s, a),
                    None => return Err(Error::AuthServerUrlInvalid(addr.to_string())),
                };

                let scheme = match scheme.parse::<authc::Scheme>() {
                    Ok(s) => s,
                    Err(_) => return Err(Error::AuthServerUrlInvalid(addr.to_string())),
                };

                let authority = match authority.parse::<authc::Authority>() {
                    Ok(a) => a,
                    Err(_) => return Err(Error::AuthServerUrlInvalid(addr.to_string())),
                };

                Ok(authc::AuthClient::new(scheme, authority)?
                    .sign_in(username, password)
                    .await?
                    .serialize())
            },
            None => Ok(username),
        },
    }
}

fn tls_client_config() -> Result<quinn::ClientConfig, Error> {
    match quinn::ClientConfig::try_with_platform_verifier() {
        Ok(config) => Ok(config),
        Err(e) => {
            warn!(?e, "Platform TLS verifier unavailable");
            #[cfg(target_os = "android")]
            return android_system_roots_config().map_err(|fallback_e| {
                warn!(?fallback_e, "Android system CA store fallback failed");
                e.into()
            });
            #[cfg(not(target_os = "android"))]
            Err(e.into())
        },
    }
}

/// Build a TLS client config from the Android system CA store.
///
/// `rustls-platform-verifier` cannot access the platform trust store on
/// Android, but the system CAs are world-readable PEM files, so load them
/// directly.
#[cfg(target_os = "android")]
fn android_system_roots_config() -> std::io::Result<quinn::ClientConfig> {
    use rustls::pki_types::{CertificateDer, pem::PemObject};
    use std::io::{Error as IoError, ErrorKind};

    let mut certs = Vec::new();
    for entry in std::fs::read_dir("/system/etc/security/cacerts")? {
        let path = entry?.path();
        let bytes = std::fs::read(&path)?;
        match CertificateDer::from_pem_slice(&bytes) {
            Ok(cert) => certs.push(cert),
            Err(e) => warn!(?path, "Skipping unreadable system CA file: {e:?}"),
        }
    }
    if certs.is_empty() {
        return Err(IoError::new(
            ErrorKind::NotFound,
            "no CA certificates in the Android system store",
        ));
    }
    let mut roots = rustls::RootCertStore::empty();
    let (added, ignored) = roots.add_parsable_certificates(certs);
    tracing::info!(added, ignored, "Loaded Android system CA certificates");
    let roots = std::sync::Arc::new(roots);
    let config = quinn::ClientConfig::with_root_certificates(roots)
        .expect("system CA roots must build a valid TLS client config");
    Ok(config)
}

pub(crate) async fn connect_quic(
    network: &Network,
    hostname: String,
    override_port: Option<u16>,
    prefer_ipv6: bool,
    validate_tls: bool,
) -> Result<network::Participant, crate::error::Error> {
    let config = if validate_tls {
        tls_client_config()?
    } else {
        warn!(
            "skipping validation of server identity. There is no guarantee that the server you're \
             connected to is the one you expect to be connecting to."
        );
        #[derive(Debug)]
        struct Verifier;
        impl rustls::client::danger::ServerCertVerifier for Verifier {
            fn verify_server_cert(
                &self,
                _end_entity: &rustls::pki_types::CertificateDer<'_>,
                _intermediates: &[rustls::pki_types::CertificateDer<'_>],
                _server_name: &rustls::pki_types::ServerName<'_>,
                _ocsp_response: &[u8],
                _now: rustls::pki_types::UnixTime,
            ) -> Result<ServerCertVerified, rustls::Error> {
                Ok(ServerCertVerified::assertion())
            }

            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &rustls::pki_types::CertificateDer<'_>,
                _dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }

            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &rustls::pki_types::CertificateDer<'_>,
                _dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }

            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                vec![
                    rustls::SignatureScheme::RSA_PKCS1_SHA1,
                    rustls::SignatureScheme::ECDSA_SHA1_Legacy,
                    rustls::SignatureScheme::RSA_PKCS1_SHA256,
                    rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                    rustls::SignatureScheme::RSA_PKCS1_SHA384,
                    rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                    rustls::SignatureScheme::RSA_PKCS1_SHA512,
                    rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
                    rustls::SignatureScheme::RSA_PSS_SHA256,
                    rustls::SignatureScheme::RSA_PSS_SHA384,
                    rustls::SignatureScheme::RSA_PSS_SHA512,
                    rustls::SignatureScheme::ED25519,
                    rustls::SignatureScheme::ED448,
                ]
            }
        }

        let mut cfg = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Verifier))
            .with_no_client_auth();
        cfg.enable_early_data = true;

        quinn::ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(cfg).unwrap(),
        ))
    };

    addr::try_connect(network, &hostname, override_port, prefer_ipv6, |a| {
        ConnectAddr::Quic(a, config.clone(), hostname.clone())
    })
    .await
}
