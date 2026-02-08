use wl_clipboard_rs::copy::{MimeType, Options, Source};

pub fn set_clipboard(value: &str) {
    dbg!("setting cilpboard");
    dbg!(value);

    let opts = Options::new();
    let result = opts.copy(
        Source::Bytes(value.to_string().into_bytes().into()),
        MimeType::Autodetect,
    );

    if let Err(e) = result {
        println!("unable to copy to clipboard {}", e);
    }
}
