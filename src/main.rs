#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin, custom_attribute)]

extern crate multipart;
#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::{self, Write};

use multipart::server::Multipart;
use multipart::server::save::Entries;
use multipart::server::save::SaveResult::*;
use rocket::Data;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;

// todo: rewrite so better structure

#[post("/upload/image", data = "<data>")]
fn multipart_upload(cont_type: &ContentType, data: Data) -> Result<String, Custom<String>> {
    if !cont_type.is_form_data() {
        return Err(Custom(Status::BadRequest, "Expected Content-Type multipart/form-data".into()));
    }

    let (_, boundary) = cont_type.params().find(|&(k, _)| k == "boundary")
        .ok_or_else(|| Custom(Status::BadRequest, "multipart/form-data boundary param not provided".into()))?;

    match process_upload(boundary, data) {
        Ok(mut resp) => {
            let mut file = match File::create("image_test.png") {
                Ok(file) => file,
                Err(why) => {
                    println!("Error occurred on File Creation! {:?}", why);
                    return Err(Custom(Status::InternalServerError, why.to_string()))
                }
            };
            let result = file.write_all(resp.as_mut_slice());
            if let Err(why) = result {
                println!("Error occurred on write all! {:?}", why);
                return Err(Custom(Status::InternalServerError, why.to_string()))
            };

            Ok(String::from("hi"))
        },
        Err(err) => Err(Custom(Status::InternalServerError, err.to_string()))
    }
}

fn process_upload(boundary: &str, data: Data) -> io::Result<Vec<u8>> {
    let mut out = Vec::new(); // todo: possible to optimize?
    let mut multipart = Multipart::with_body(data.open(), boundary);

    match multipart.save().size_limit(10000000).temp() { // todo: verify whether 10mb is too much
        Full(entries) => process_entries(entries, &mut out)?,
        Partial(partial, reason) => {
            println!("Save operation quit unexpectedly! {:?}", reason);
            process_entries(partial.entries, &mut out)? // todo: make this fail! only upload images if the operation was fully successful
        },
        Error(e) => return Err(e),
    }

    Ok(out)
}

fn process_entries(entries: Entries, out: &mut Vec<u8>) -> io::Result<()> {
    // todo: check whether this can be easily optimized
    for fields in entries.fields.values() {
        for field in fields.iter() {
            let mut data = field.data.readable()?;
            io::copy(&mut data, out)?;
        }
    }

    Ok(())
}

fn main() {
    rocket::ignite().mount("/api/v1", routes![multipart_upload]).launch();
}