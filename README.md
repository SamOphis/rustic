# Rustic
Rustic is a personal media server made for two reasons:
* to get familiar with [Rust](https://www.rust-lang.org).
* to get familiar with writing server applications.
* to deal with [Flameshot](https://flameshot.js.org/#/) custom image uploads.
    * note: flameshot custom uploader should be added here or in my [dotfiles](https://github.com/SamOphis/dotfiles).
    * note: [ShareX](https://github.com/ShareX/ShareX) compatibility is not a goal in mind as of writing this.

# Usage in Production
This is absolutely by no-means a production-ready project. There are many other similar projects which are
much more mature and stable. Please use those instead.

# Details
This project is written in [Rust](https://www.rust-lang.org) using the [Rocket](https://github.com/SergioBenitez/Rocket)
web framework. Rustic can be built the same way any other Cargo project can, via. `cargo build`. You can specify the
`--release` option to apply many different optimizations. The project can be run with `cargo run`, however
this is unnecessary if using a pre-built release.

## First Iteration
The first iteration of this project is just a generic media server. It accepts `multipart/form-data` uploads, with
each entry being limited to 10MB. This limit is unconfigurable as of version 0.3.0. When uploading images, the
file name is returned (without the extension as of 0.3.0). This name can be used to fetch the media source you uploaded.

Media will never expire unless manually deleted, and all media is assigned a unique random base64-encoded ID. This ID is
an unsigned 128-bit integer, and the maximum bound can be configured with the `MAX_FILE_ID` environment variable,
which defaults to `64^8`. Media is stored in a directory specified by the optional environment variable `MEDIA_DIRECTORY`,
which defaults to `media/`.

All environment variables are processed lazily. An `AUTHORIZATION` environment variable ***must*** be present
for any authorization-requiring request to be performed.

> **Note:** As of 0.3.0, when specifying a custom media directory, the name **must** end with a `/`.

