#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin, custom_attribute)]

extern crate multipart;
#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::{self, Write};

use multipart::server::Multipart;
use multipart::server::save::SaveResult::*;
use rocket::Data;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;

#[post("/upload/image", data = "<data>")]
fn image_upload(content_type: &ContentType, data: Data) -> Result<Status, Custom<String>> {
    // the following checks can be implemented with rocket request guards but I despise them.
    if !content_type.is_form_data() {
        return Err(Custom(Status::BadRequest, "Expected Content-Type multipart/form-data".into()));
    }

    let (_, boundary) = content_type.params().find(|&(key, _)| key == "boundary")
        .ok_or_else(|| Custom(Status::BadRequest, "multipart/form-data boundary parameter not provided".into()))?;

    let mut output = Vec::new();
    let mut multipart = Multipart::with_body(data.open(), boundary);
    match multipart.save().size_limit(10000000).temp() {
        Full(entries) => {
            for fields in entries.fields.values() {
                for field in fields.iter() {
                    let mut data = match field.data.readable() {
                        Ok(data) => data,
                        Err(error) => {
                            println!("Error fetching readable data! {:?}", error);
                            return Err(Custom(Status::InternalServerError, error.to_string()))
                        }
                    };
                    if let Err(error) = io::copy(&mut data, &mut output) {
                        return Err(Custom(Status::InternalServerError, error.to_string()))
                    }
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
    };

    let mut file = match File::create("image_test.png") {
        Ok(file) => file,
        Err(error) => {
            println!("Error occurred on File Creation! {:?}", error);
            return Err(Custom(Status::InternalServerError, error.to_string()))
        }
    };
    match file.write_all(output.as_mut_slice()) {
        Ok(()) => Ok(Status::NoContent),
        Err(error) => Err(Custom(Status::InternalServerError, error.to_string()))
    }
}

fn main() {
    rocket::ignite().mount("/api/v1", routes![image_upload]).launch();
}