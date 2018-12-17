extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate chrono;
use quicli::prelude::*;
use structopt::StructOpt;
use std::fs::File;
use std::io::prelude::*;
use chrono::prelude::*;

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
}

fn main() -> CliResult {
    let args = Cli::from_args();

    let base_url = "https://licenz.zbrox.com/";
    let license_text_base_url = format!("{}{}", base_url, "license_text/");

    let selected_license = args.license;
    let body = reqwest::get(base_url)?.text()?;
    let licenses: Vec<License> = serde_json::from_str(&body.to_owned())?;

    for license in licenses.iter() {
       if license.key == selected_license {
            println!("Selected license: {}", license.name);
            let license_url = format!("{}{}.txt", license_text_base_url, license.key);
            
            println!("Downloading license...");
            let license_body = reqwest::get(&license_url)?.text()?;

            write_file(
                fill_in_details(&license_body, &args.copyright_holder),
                &args.filename
            )?;
            println!("License saved in LICENSE");
            
            break;
       }
    }

    Ok(())
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