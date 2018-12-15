#![allow(clippy::type_complexity)]

#[macro_use]
extern crate failure_derive;

mod filesystem;
mod ioutil;
mod mapping;
mod mp3;

use self::filesystem::*;
use self::mapping::*;
use log::*;
use std::process;

fn main() {
    env_logger::init();

    let cli = clap::App::new("SoundCloud FS")
        .version("0.1.0")
        .author("polyfloyd <floyd@polyfloyd.net>")
        .about("A FUSE driver for SoundCloud audio")
        .arg(
            clap::Arg::with_name("path")
                .short("p")
                .long("path")
                .takes_value(true)
                .required(true)
                .help("Sets the target directory of the mount"),
        ).arg(
            clap::Arg::with_name("user")
                .short("u")
                .long("user")
                .takes_value(true)
                .required(true)
                .help("Sets the user to create directory and file entries for"),
        ).arg(
            clap::Arg::with_name("login")
                .long("login")
                .value_name("username:password")
                .takes_value(true)
                .validator(|s| match s.splitn(2, ':').count() {
                    2 => Ok(()),
                    c => Err(format!("bad credential format, split on : yields {} strings", c)),
                }).help("Logs in using a username and password instead of accessing the API anonymously"),
        ).arg(
            clap::Arg::with_name("id3-images")
                .long("id3-images")
                .value_name("enable")
                .takes_value(true)
                .default_value("0")
                .possible_values(&["0", "1"])
                .help("Enables image metadata in ID3 tags. This will incur an additional HTTP request everytime a file is opened for reading"),
        ).get_matches();

    let sc_config = soundcloud::Config {
        id3_download_images: cli.value_of("id3-images") == Some("1"),
    };

    let login = cli.value_of("login").and_then(|s| {
        let mut i = s.splitn(2, ':');
        let u = i.next().unwrap();
        i.next().map(|p| (u, p))
    });
    let sc_client_rs = match login {
        None => {
            info!("creating anonymous client");
            soundcloud::Client::anonymous(sc_config)
        }
        Some((username, password)) => {
            info!("logging in as {}", username);
            soundcloud::Client::login(sc_config, &username, password)
        }
    };

    let sc_client = match sc_client_rs {
        Ok(v) => v,
        Err(err) => {
            error!("could not initialize SoundCloud client: {}", err);
            process::exit(1);
        }
    };

    let username = cli.value_of("user").unwrap();
    let root = Root {
        sc_client: &sc_client,
        username: username.to_string(),
    };
    let fs = FS::new(&CacheRoot::new(&root));
    let path = cli.value_of("path").unwrap();
    fuse::mount(fs, &path, &[]).unwrap();
}
