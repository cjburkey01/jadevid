# JadeVid

A video editor...I'm trying again...Maybe I'll get somewhere ¯\\_(ツ)_/¯

### How do I build this?

* Check [how to build the ffmpeg-next crate](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building)
  for your operating system.
* Check the list of features enabled for ffmpeg-next in [this project's Cargo.toml](./Cargo.toml)
  * You can remove certain build features if you don't need those formats, like `build-lib-mp3lame`
    for the mp3 format, as an example.
  * You can remove the "build" feature and associated "build-" features entirely to avoid building
    ffmpeg itself
  * The current Cargo.toml builds successfully on my Intel Mac with ffmpeg@7 installed via Homebrew
