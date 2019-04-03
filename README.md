# What is this?

This is a small and simple command line tool which can help you add a license file to your repo and can be useful when automating that task.
As an additional functionality you can compare files on your disk with given licenses. This is to see if they actually match the expected license or if they differ and to what extend.

# How do I run this?

You need to have [cargo](https://github.com/rust-lang/cargo/) installed and then just type `cargo run` in the same
folder where `Cargo.toml` is located.

This utility is also published on [crates.io](https://crates.io/crates/licenz).

# How do I use this?

This utility has a simple enough `--help` argument which will print out what you can do with it.

Here's a simple example which will download the MIT license: 

    licenz download --license mit --copyright "Jane Doe"

Here's a simple example which will compare the `LICENSE` file in the current folder with the MIT license:
    
    licenz verify --license mit --copyright "Jane Doe"

# Where does it get the licenses?

They are served as static files on a server I'm running. The "backend" files are in the `backend`
folder here which consist of the text of the license files and a json file describing them.

Everything you need to do in order to duplicate this is to run a simple web server, edit
`main.rs` to point to the new location, and then `cargo build`.

Oh and yeah, any suggestions on omitted licenses are welcome.