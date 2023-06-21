use anyhow::Error;
use futures_util::{future::IntoStream, FutureExt};
use hyper::{
    client::{self, HttpConnector, ResponseFuture},
    header::RANGE,
    Request,
};
use hyper_rustls::HttpsConnector;
use rustls::{OwnedTrustAnchor, RootCertStore};

struct NoCertificateVerification {}

impl rustls::client::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
pub struct Client {
    client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        let mut root_store = RootCertStore::empty();
        root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let tls = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Prepare the HTTPS connector
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .build();
        let client = client::Client::builder().build(https);
        Self { client }
    }

    pub fn stream_from_url(
        &self,
        url: &str,
        offset: usize,
        length: usize,
    ) -> Result<IntoStream<ResponseFuture>, Error> {
        let req = Request::builder()
            .method("GET")
            .uri(url)
            .header(RANGE, format!("bytes={}-{}", offset, offset + length - 1))
            .body(hyper::Body::empty())?;
        Ok(self.client.request(req).into_stream())
    }
}
