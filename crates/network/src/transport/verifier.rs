//! QUIC client certificate verifier.

use rustls::{
    client::danger::HandshakeSignatureValid,
    client::danger::{ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
    DigitallySignedStruct, DistinguishedName, SignatureScheme,
};
use webpki::{ring, EndEntityCert, KeyUsage};

/// Verifier for QUIC connections.
#[derive(Debug)]
pub struct Verifier;

impl Verifier {
    fn extract_public_key(cert: &EndEntityCert) -> Result<[u8; 32], rustls::Error> {
        let spki = cert.subject_public_key_info();

        // The DER encoding should be 44 bytes for ED25519:
        // SEQUENCE (2 bytes) + AlgorithmIdentifier (5 bytes) + BIT STRING tag (2 bytes) + 32 bytes key
        if spki.len() != 44 {
            tracing::warn!("Unexpected SubjectPublicKeyInfo length: {}", spki.len());
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
        intermediates: &[CertificateDer<'_>],
        now: UnixTime,
        key_usage: KeyUsage,
    ) -> Result<[u8; 32], rustls::Error> {
        // Get and verify the DNS name
        let Some(alt) = cert.valid_dns_names().next() else {
            tracing::warn!("No DNS name found in certificate");
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::NotValidForName,
            ));
        };

        // Extract and verify the public key matches the DNS name
        let bytes = Self::extract_public_key(cert)?;
        let encoded = base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &bytes);
        if alt.len() != 53 || !alt.starts_with("e") || alt[1..] != encoded {
            tracing::warn!("DNS name mismatch: expected e{}, got {}", encoded, alt);
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::NotValidForName,
            ));
        }

        // Verify the certificate usage
        cert.verify_for_usage(
            &[ring::ED25519],
            &[], // Skip trust anchor verification since we're using self-signed certs
            intermediates,
            now,
            key_usage,
            None,
            None,
        )
        .map_err(|e| {
            tracing::warn!("Certificate usage verification failed: {:?}", e);
            pki_error(e)
        })?;

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
        cert.verify_signature(ring::ED25519, message, dss.signature())
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
