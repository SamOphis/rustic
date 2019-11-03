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
The first iteration of this project is just a generic image server. When paired with a compatible uploader executable,
an image is uploaded to a given site (when authorized) and gets given a unique identifier and URL to accompany it,
which can then be embedded anywhere for any amount of time. Images will never expire unless manually deleted.

**Note:** Media files, so far, are PNG images with base64 random ID's. ID's are bounded to 64^8 which is more images
than you could ever hope of exhausting. They're stored in a `media` directory which must be created in the application
working directory beforehand. Future iterations/versions of Rustic will be way more configurable and automated, with
more media types supported too (video, audio, etc.)

