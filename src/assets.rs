use include_directory::{Dir, File, include_directory};
use rocket::{Route, get, http::ContentType, routes};
use std::path::Path;

#[get("/<asset>")]
pub fn assets(asset: &str) -> (ContentType, Vec<u8>) {
    let file = read_any_file(asset);
    let bytes = file.contents();

    let file_type = asset.split('.').last().unwrap();

    let ct: ContentType = match file_type {
        "js" => ContentType::JavaScript,
        "css" => ContentType::CSS,
        "html" => ContentType::HTML,
        "png" => ContentType::PNG,
        "svg" => ContentType::SVG,
        "json" => ContentType::JSON,
        "xml" => ContentType::XML,
        "msgpack" => ContentType::MsgPack,
        "txt" => ContentType::Plain,
        "ico" => ContentType::Icon,
        _ => {
            return (
                ContentType::Plain,
                "Unexpected type requested".as_bytes().to_vec(),
            );
        }
    };
    (ct, bytes.to_vec())
}

pub fn api() -> (&'static str, Vec<Route>) {
    ("/_assets", routes![assets])
}

static PROJECT_DIR: Dir<'_> = include_directory!("assets");

pub fn read_any_file(name: &str) -> File {
    let path = Path::new(name);
    let file = PROJECT_DIR
        .get_file(path)
        .unwrap_or_else(|| panic!("could not find file this name: {}", path.to_str().unwrap()));
    file.clone()
}
