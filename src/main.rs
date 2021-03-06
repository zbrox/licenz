extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate exitcode;

use quicli::prelude::*;
use structopt::StructOpt;
use std::fs::File;
use std::io::prelude::*;
use chrono::prelude::*;
use std::path::Path;
use human_panic::{setup_panic};
use colored_diff;
use colored::*;

const BASE_URL: &str = "https://licenz.zbrox.com/";

#[derive(Deserialize, Clone)]
struct License {
    key: String,
    name: String,
}

#[derive(Debug, StructOpt)]
/// Put a LICENSE file in the current directory with the text of your license of choice
enum Cli {
    /// Subcommand for fetching a license
    #[structopt(name = "download")]
    Download(Download),

    /// Subcommand for verifying licenses
    #[structopt(name = "verify")]
    Verify(Verify),
}

#[derive(Debug, StructOpt)]
struct Download {
    /// List available license keys you can use for the --license argument
    #[structopt(long = "list")]
    list: bool,

    /// Which license to add
    #[structopt(long = "license", short = "l", required_unless = "list")]
    license: Option<String>,

    /// The name of the copyright holder be it organization or a person
    #[structopt(long = "copyright", short = "c", required_unless = "list")]
    copyright_holder: Option<String>,

    /// The file in which to save the license text
    #[structopt(long = "file", short = "f", default_value = "LICENSE")]
    filename: String,

    /// Overwrite the LICENSE file if it already exists
    #[structopt(long = "overwrite", short = "o")]
    overwrite: bool,
}

#[derive(Debug, StructOpt)]
struct Verify {
    /// Which license to compare against
    #[structopt(long = "license", short = "l", required_unless = "list")]
    license: Option<String>,

    /// The file in which you have a license saved in for comparison
    #[structopt(long = "file", short = "f", default_value = "LICENSE")]
    filename: String,

    /// The name of the copyright holder be it organization or a person
    #[structopt(long = "copyright", short = "c", required_unless = "list")]
    copyright_holder: Option<String>,
}

fn download_subcommand(args: Download) -> CliResult {
    if args.list {
        let key_list = get_license_keys()?;
        println!("Available licenses: {}", key_list);
        return Ok(());
    }

    if !args.overwrite && Path::new(&args.filename).exists() {
        println!("File {} already exists. If you wanna overwrite it pass the -o option.", &args.filename);
        std::process::exit(exitcode::DATAERR);
    }

    let selected_license: String = match args.license {
        Some(l) => l,
        None => {
            println!("--license not specified");
            std::process::exit(exitcode::DATAERR);
        }
    };

    let copyright_holder: String = match args.copyright_holder {
        Some(c) => c,
        None => {
            println!("--copyright not specified");
            std::process::exit(exitcode::DATAERR);
        }
    };

    let license = match get_license_by_key(&selected_license)? {
        Some(l) => l,
        None => {
            println!("Selected license {} not found", selected_license);
            std::process::exit(exitcode::DATAERR);
        }
    };

    println!("Selected license: {}", license.name);
    println!("Downloading license...");
    let license_body = download_license_text(&license)?;

    write_file(
        fill_in_details(&license_body, &copyright_holder),
        &args.filename
    )?;
    println!("License saved in LICENSE");

    Ok(())
}

fn main() -> CliResult {
    setup_panic!();
    let args = Cli::from_args();

    match args {
        Cli::Download(v) => download_subcommand(v),
        Cli::Verify(v) => compare(v),
    }
}

fn get_license_by_key(key: &str) -> Result<Option<License>, Error> {
    let licenses: Vec<License> = get_licenses()?;

    let license = licenses.into_iter().find(|l| l.key == key);

    Ok(license)
}

fn get_license_keys() -> Result<String, Error> {
    let licenses = get_licenses()?;
    
    let keys: Vec<String> = licenses
        .into_iter()
        .map(|l| l.key)
        .collect();

    Ok(keys.join(", "))
}

fn download_license_text(license: &License) -> Result<String, Error> {
    let license_url = get_license_text_url(&license);
    let body = reqwest::get(&license_url)?.text()?;

    Ok(body)
}

fn get_licenses() -> Result<Vec<License>, Error> {
    let body = reqwest::get(BASE_URL)?.text()?;
    let licenses: Vec<License> = serde_json::from_str(&body.to_owned())?;
    Ok(licenses)
}

fn get_license_text_url(license: &License) -> String {
    let license_text_base_url = format!("{}{}", BASE_URL, "license_text/");
    return format!("{}{}.txt", license_text_base_url, license.key);
}

fn fill_in_details(license_body: &str, copyright_holder: &str) -> String {
    let date = Local::now();
    let year_string = format!("{}", date.year());
    let text_with_year = license_body.replace("<YEAR>", &year_string);

    text_with_year.replace("<COPYRIGHT_HOLDER>", copyright_holder)
}

fn write_file(text: String, filename: &str) -> Result<(), Error> {
    let mut buffer = File::create(filename)?;
    buffer.write_all(&text.into_bytes())?;
    Ok(())
}

fn compare(args: Verify) -> CliResult {
    let selected_license: String = match args.license {
        Some(l) => l,
        None => {
            println!("--license not specified");
            std::process::exit(exitcode::DATAERR);
        }
    };

    let copyright_holder: String = match args.copyright_holder {
        Some(c) => c,
        None => {
            println!("--copyright not specified");
            std::process::exit(exitcode::DATAERR);
        }
    };

    let license = match get_license_by_key(&selected_license)? {
        Some(l) => l,
        None => {
            println!("Selected license {} not found", selected_license);
            std::process::exit(exitcode::DATAERR);
        }
    };

    let license_body = fill_in_details(&download_license_text(&license)?, &copyright_holder);
    let license_body_on_disk = read_file(&args.filename)?;

    if license_body == license_body_on_disk {
        println!("{} seems to match the one saved on disk in {}", &license.name.red(), &args.filename.green());
        return Ok(());
    }

    let diff = colored_diff::PrettyDifference { 
        expected: &license_body, 
        actual: &license_body_on_disk 
    };

    println!("Here are the differences found between the file {} and the {} contents", &args.filename.green(), &license.name.red());
    println!("{}", diff);

    Ok(())
}