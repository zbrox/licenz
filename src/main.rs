extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate chrono;
use quicli::prelude::*;
use structopt::StructOpt;
use std::fs::File;
use std::io::prelude::*;
use chrono::prelude::*;
use std::path::Path;

const BASE_URL: &str = "https://licenz.zbrox.com/";

#[derive(Deserialize)]
struct License {
    key: String,
    name: String,
}

#[derive(Debug, StructOpt)]
/// Put a LICENSE file in the current directory with the text of your license of choice
struct Cli {
    /// Which license to add
    #[structopt(long = "license", short = "l")]
    license: String,

    /// The name of the copyright holder be it organization or a person
    #[structopt(long = "copyright", short = "c")]
    copyright_holder: String,

    /// The file in which to save the license text
    #[structopt(long = "file", short = "f", default_value = "LICENSE")]
    filename: String,

    /// Overwrite the LICENSE file if it already exists
    #[structopt(long = "overwrite", short = "o")]
    overwrite: bool,
}

fn main() -> CliResult {
    let args = Cli::from_args();

    if !args.overwrite && Path::new(&args.filename).exists() {
        println!("File {} already exists. If you wanna overwrite it pass the -o option.", &args.filename);
        return Ok(());
    }

    let licenses: Vec<License> = get_licenses()?;

    let selected_license = args.license;
    let mut found: bool = false;

    for license in licenses.iter() {
       if license.key == selected_license {
            found = true;
            println!("Selected license: {}", license.name);
            
            println!("Downloading license...");
            let license_body = download_license_text(license)?;

            write_file(
                fill_in_details(&license_body, &args.copyright_holder),
                &args.filename
            )?;
            println!("License saved in LICENSE");
            
            break;
       }
    }

    if !found {
        println!("Selected license {} not found", selected_license);
    }

    Ok(())
}

fn download_license_text(license: &License) -> Result<String, Error> {
    let license_url = get_license_text_url(&license);
    let body = reqwest::get(&license_url)?.text()?;

    return Ok(body);
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

    return text_with_year.replace("<COPYRIGHT_HOLDER>", copyright_holder);
}

fn write_file(text: String, filename: &str) -> Result<(), Error> {
    let mut buffer = File::create(filename)?;
    buffer.write(&text.into_bytes())?;
    Ok(())
}