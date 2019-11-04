#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin, custom_attribute)]

extern crate multipart;
#[macro_use]
extern crate rocket;

use std::env::var;
use std::fs::{create_dir, File, metadata};
use std::io::copy;
use std::path::PathBuf;

use base64::{encode_config, URL_SAFE_NO_PAD};
use multipart::server::Multipart;
use multipart::server::save::SaveResult::*;
use once_cell::sync::Lazy;
use rand::Rng;
use rocket::Data;
use rocket::http::{ContentType, Status};
use rocket::response::NamedFile;
use rocket::response::status::Custom;

static MEDIA_DIRECTORY: Lazy<String> = Lazy::new(|| {
    let dir = match var("MEDIA_DIRECTORY") {
        Ok(dir) => dir,
        Err(error) => {
            println!("Error occurred while obtaining MEDIA_DIRECTORY env var. Defaulting to media/ | {:?}", error);
            String::from("media/")
        }
    };

    if metadata(&dir).is_err() {
        if let Err(error) = create_dir(&dir) {
            panic!("Failure to open/create {} directory! {:?}", dir, error)
        }
    }

    dir
});

static MAX_FILE_ID: Lazy<u128> = Lazy::new(|| {
    match var("MAX_FILE_ID") {
        Ok(max) => match max.trim().parse::<u128>() {
            Ok(max) => max,
            Err(error) => {
                println!("Error when parsing MAX_FILE_ID into u128. Defaulting to 64^8. {:?}", error);
                281474976710656
            }
        },
        Err(error) => {
            println!("Error occurred while obtaining MAX_FILE_ID env var. Defaulting to 64^8. {:?}", error);
            281474976710656
        }
    }
});

fn retrieve_new_file() -> std::io::Result<(PathBuf, File)> {
    loop {
        let rand = rand::thread_rng().gen_range(0, *MAX_FILE_ID);
        let bytes = rand.to_le_bytes();

        let mut last_non_zero = 15;
        while last_non_zero > 0 && bytes[last_non_zero] == 0 {
            last_non_zero = last_non_zero - 1;
        }

        let id = encode_config(&bytes[..last_non_zero], URL_SAFE_NO_PAD);
        let mut path_buf = PathBuf::from(&*MEDIA_DIRECTORY);
        path_buf.push(&id);

        if !path_buf.exists() {
            return File::create(&path_buf)
                .map(|file| (path_buf, file))
        }
    };
}

#[get("/images/<id>")]
fn get_image(id: String) -> Result<NamedFile, Status> {
    let mut path_buf = PathBuf::from(&*MEDIA_DIRECTORY);
    path_buf.push(id);

    NamedFile::open(path_buf)
        .map_err(|_| Status::NotFound)
}

#[post("/upload/image", data = "<data>")]
fn image_upload(content_type: &ContentType, data: Data) -> Result<String, Custom<String>> {
    // the following checks can be implemented with rocket request guards but I despise them.
    if !content_type.is_form_data() {
        return Err(Custom(Status::BadRequest, "Expected Content-Type multipart/form-data".into()));
    }

    let (_, boundary) = content_type.params().find(|&(key, _)| key == "boundary")
        .ok_or_else(|| Custom(Status::BadRequest, "multipart/form-data boundary parameter not provided".into()))?;

    let mut multipart = Multipart::with_body(data.open(), boundary);
    match multipart.save().size_limit(10000000).temp() {
        Full(entries) => {
            let (path_buf, mut file) = match retrieve_new_file() {
                Ok(tuple) => tuple,
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

            return match path_buf.file_name() {
                None => {
                    println!("An error occurred when fetching file name of path buffer.");
                    Err(Custom(Status::InternalServerError, "Failure to obtain filename.".into()))
                },
                Some(os_str) => match os_str.to_str() {
                    None => {
                        println!("An error occurred when converting &OsStr to &str. Invalid unicode?");
                        Err(Custom(Status::InternalServerError, "Failure to obtain filename.".into()))
                    },
                    Some(str) => Ok(String::from(str))
                }
            }
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
    rocket::ignite()
        .mount("/api/v1", routes![image_upload])
        .mount("/", routes![get_image])
        .launch();
}