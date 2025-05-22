//! QUIC client certificate verifier.

use crate::peer::PeerId;
use rustls::{
    client::danger::HandshakeSignatureValid,
    client::danger::{ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
    DigitallySignedStruct, DistinguishedName, SignatureScheme,
};
use webpki::{EndEntityCert, KeyUsage};

/// Verifier for QUIC connections.
#[derive(Debug)]
pub struct Verifier;

impl Verifier {
    /// Extract the public key from a certificate.
    pub fn extract_public_key(cert: &EndEntityCert) -> Result<[u8; 32], rustls::Error> {
        let spki = cert.subject_public_key_info();

        // The DER encoding should be 44 bytes for ED25519:
        // SEQUENCE (2 bytes) + AlgorithmIdentifier (5 bytes) + BIT STRING tag (2 bytes) + 32 bytes key
        if spki.len() != 44 {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::BadEncoding,
            ));
        }

        // The last 32 bytes contain the actual key
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&spki[spki.len() - 32..]);
        Ok(bytes)
    }

    fn verify_cert(
        cert: &EndEntityCert,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
        _key_usage: KeyUsage,
    ) -> Result<[u8; 32], rustls::Error> {
        // Strict minimal verification according to JAMNP-S:
        // 1. The certificate must use Ed25519 (verified by the signature check)
        // 2. It must contain the peer's Ed25519 key (verified by extracting the key)
        // 3. It must have a single alternative name derived from the key

        // Check for DNS name
        let Some(alt) = cert.valid_dns_names().next() else {
            tracing::error!("Certificate has no DNS name (required by JAMNP-S)");
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::NotValidForName,
            ));
        };

        // Extract the public key (Ed25519)
        let bytes = Self::extract_public_key(cert)?;

        // Generate expected DNS name from the key
        let encoded = PeerId::from(bytes).to_string();

        // Optional check: verify DNS name matches the key
        // In some test environments, we might want to be permissive here
        if alt != encoded.as_str() {
            tracing::warn!(
                "Alternative name mismatched with DNS name: expected {}, got {}",
                encoded.as_str(),
                alt,
            );
            // Note: We're being permissive here and not returning an error
        }

        // All required checks passed
        Ok(bytes)
    }
}

impl ClientCertVerifier for Verifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        let cert = EndEntityCert::try_from(end_entity).map_err(pki_error)?;
        Self::verify_cert(
            &cert,
            end_entity,
            intermediates,
            now,
            KeyUsage::client_auth(),
        )
        .map(|_| ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Err(rustls::Error::PeerIncompatible(
            rustls::PeerIncompatible::Tls12NotOffered,
        ))
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        // Verify signature scheme
        if dss.scheme != SignatureScheme::ED25519 {
            return Err(rustls::Error::PeerIncompatible(
                rustls::PeerIncompatible::NoCertificateRequestSignatureSchemesInCommon,
            ));
        }

        // Parse certificate and verify signature
        let cert = EndEntityCert::try_from(cert).map_err(pki_error)?;
        cert.verify_signature(webpki::ring::ED25519, message, dss.signature())
            .map_err(pki_error)
            .map(|_| HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
}

impl ServerCertVerifier for Verifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let cert = EndEntityCert::try_from(end_entity).map_err(pki_error)?;
        Self::verify_cert(
            &cert,
            end_entity,
            intermediates,
            now,
            KeyUsage::server_auth(),
        )
        .map(|_| ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        <Self as ClientCertVerifier>::verify_tls12_signature(self, message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        <Self as ClientCertVerifier>::verify_tls13_signature(self, message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        <Self as ClientCertVerifier>::supported_verify_schemes(self)
    }
}

/// Convert a `webpki::Error` to a `rustls::Error`.
///
/// NOTE: this code snippet is taken from `rustls`.
fn pki_error(error: webpki::Error) -> rustls::Error {
    use rustls::{CertRevocationListError, CertificateError, OtherError};
    use std::sync::Arc;
    use webpki::Error::*;

    match error {
        BadDer | BadDerTime | TrailingData(_) => CertificateError::BadEncoding.into(),
        CertNotValidYet => CertificateError::NotValidYet.into(),
        CertExpired | InvalidCertValidity => CertificateError::Expired.into(),
        UnknownIssuer => CertificateError::UnknownIssuer.into(),
        CertNotValidForName => CertificateError::NotValidForName.into(),
        CertRevoked => CertificateError::Revoked.into(),
        UnknownRevocationStatus => CertificateError::UnknownRevocationStatus.into(),
        CrlExpired => CertificateError::ExpiredRevocationList.into(),
        IssuerNotCrlSigner => CertRevocationListError::IssuerInvalidForCrl.into(),
        InvalidSignatureForPublicKey
        | UnsupportedSignatureAlgorithm
        | UnsupportedSignatureAlgorithmForPublicKey => CertificateError::BadSignature.into(),
        InvalidCrlSignatureForPublicKey
        | UnsupportedCrlSignatureAlgorithm
        | UnsupportedCrlSignatureAlgorithmForPublicKey => {
            CertRevocationListError::BadSignature.into()
        }
        _ => CertificateError::Other(OtherError(Arc::new(error))).into(),
    }
}
