#[derive(serde::Serialize, serde::Deserialize)]
struct Foo {
    #[serde(with = "jamcodec")]
    bar: [u8; 64],
}
