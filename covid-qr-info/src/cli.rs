use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgGroup, ArgMatches,
};

pub(super) fn get_matches() -> ArgMatches<'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .args(
            &[
                Arg::with_name("auto")
                    .index(1)
                    .help("autodetect file type")
                    .takes_value(true),
                Arg::with_name("pdf")
                    .short("p")
                    .long("pdf")
                    .help("read PDF file")
                    .takes_value(true),
                Arg::with_name("image")
                    .short("i")
                    .long("image")
                    .help("read QR code from image")
                    .takes_value(true),
                Arg::with_name("base64")
                    .short("b")
                    .long("base64")
                    .help("read base64-encoded payload")
                    .takes_value(true),
                Arg::with_name("encrypted")
                    .short("e")
                    .long("encrypted")
                    .help("read encrypted binary payload")
                    .takes_value(true),
                Arg::with_name("record")
                    .short("r")
                    .long("plaintext")
                    .help("read plaintext record")
                    .takes_value(true),
            ][..],
        )
        .group(
            ArgGroup::with_name("input_type")
                .args(&["pdf", "image", "base64", "encrypted", "record", "auto"])
                .required(true),
        )
        .get_matches()
}
