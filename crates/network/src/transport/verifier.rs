//! QUIC client certificate verifier.

use rustls::{
    client::danger::HandshakeSignatureValid,
    pki_types::{CertificateDer, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
    DigitallySignedStruct, DistinguishedName, SignatureScheme,
};
use webpki::{ring, EndEntityCert, KeyUsage};

/// Verifier for QUIC connections.
#[derive(Debug)]
pub struct Verifier;

impl ClientCertVerifier for Verifier {
    // TODO: mb hint with spacejam info, but need to figure
    // out how to create DN elegantly ...
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

        // parse the DNS name
        let Some(alt) = cert.valid_dns_names().next() else {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::NotValidForName,
            ));
        };

        // check the DNS name
        let mut bytes = [0; 32];
        bytes.copy_from_slice(&cert.subject_public_key_info());

        let encoded = base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &bytes);
        if alt.len() != 53 || !alt.starts_with("e") || alt[1..] != encoded {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::NotValidForName,
            ));
        }

        cert.verify_for_usage(
            &[ring::ED25519],
            &[],
            intermediates,
            now,
            KeyUsage::client_auth(),
            None,
            None,
        )
        .map_err(pki_error)
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
        if dss.scheme != SignatureScheme::ED25519 {
            return Err(rustls::Error::PeerIncompatible(
                rustls::PeerIncompatible::NoCertificateRequestSignatureSchemesInCommon,
            ));
        }

        let cert = EndEntityCert::try_from(cert).map_err(pki_error)?;
        cert.verify_signature(ring::ED25519, message, dss.signature())
            .map_err(pki_error)
            .map(|_| HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
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
