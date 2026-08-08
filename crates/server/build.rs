fn main() {
    wtransport::tls::Certificate::from_der(include_bytes!("cert.der").into())
        .expect("certificate DER is malformed");
}
