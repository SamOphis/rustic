#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin, custom_attribute)]

extern crate multipart;
#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

use base64::{CharacterSet, Config, encode_config, URL_SAFE, URL_SAFE_NO_PAD};
use multipart::server::Multipart;
use multipart::server::save::SaveResult::*;
use rand::Rng;
use rocket::Data;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;

const MAX_FILE_ID: u64 = 281474976710656;
// 64^8
const MEDIA_DIRECTORY: &'static str = "media/";

fn retrieve_new_file() -> std::io::Result<File> {
    loop {
        let rand = rand::thread_rng().gen_range(0, MAX_FILE_ID);
        let bytes = rand.to_le_bytes();

        let mut last_non_zero = 7;
        while last_non_zero > 0 && bytes[last_non_zero] == 0 {
            last_non_zero = last_non_zero - 1;
        }

        let mut id = encode_config(&bytes[..last_non_zero], URL_SAFE_NO_PAD);
        id.push_str(".png");

        let mut path_buf = PathBuf::from(MEDIA_DIRECTORY);
        path_buf.push(&id);

        if !path_buf.exists() {
            println!("{:?}", path_buf);
            return File::create(path_buf)
        }
    };
}

#[post("/upload/image", data = "<data>")]
fn image_upload(content_type: &ContentType, data: Data) -> Result<Status, Custom<String>> {
    // the following checks can be implemented with rocket request guards but I despise them.
    if !content_type.is_form_data() {
        return Err(Custom(Status::BadRequest, "Expected Content-Type multipart/form-data".into()));
    }

    let (_, boundary) = content_type.params().find(|&(key, _)| key == "boundary")
        .ok_or_else(|| Custom(Status::BadRequest, "multipart/form-data boundary parameter not provided".into()))?;

    let mut multipart = Multipart::with_body(data.open(), boundary);
    match multipart.save().size_limit(10000000).temp() {
        Full(entries) => {
            let mut file = match retrieve_new_file() {
                Ok(file) => file,
                Err(error) => {
                    println!("Error occurred on File Creation! {:?}", error);
                    return Err(Custom(Status::InternalServerError, error.to_string()))
                }
            };

            for fields in entries.fields.values() {
                for field in fields.iter() {
                    let mut data = match field.data.readable() {
                        Ok(data) => data,
                        Err(error) => {
                            println!("Error fetching readable data! {:?}", error);
                            return Err(Custom(Status::InternalServerError, error.to_string()))
                        }
                    };
                    if let Err(error) = copy(&mut data, &mut file) {
                        println!("Error occurred while copying data to file! {:?}", error);
                        return Err(Custom(Status::InternalServerError, error.to_string()))
                    }
                }
            }

            return Ok(Status::NoContent)
        },
        Partial(_, reason) => {
            println!("Operation quit unexpectedly while processing a file upload! {:?}", reason);
            return Err(Custom(Status::InternalServerError, format!("{:?}", reason)))
        },
        Error(error) => {
            println!("An error occurred while processing a file upload! {:?}", error);
            return Err(Custom(Status::InternalServerError, error.to_string()))
        }
    }
}

fn main() {
    rocket::ignite().mount("/api/v1", routes![image_upload]).launch();
}