use std::env;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use actix_web::client::{Client, ClientBuilder, Connector};
use actix_web::Result;
use dotenv::dotenv;
use regex::Regex;

// const LAST_SEEN_REG: &str = r"^(20|19[987])";
const LAST_SEEN_REG: &str = r"^.*";
const CATEGORY_REG: &str = r"^Extinct";
const MAX_DB_SIZE: usize = 100;

const BODY_SIZE: usize = 1024 * 1024 * 1024; // 1GB
const REQ_TIMEOUT: Duration = Duration::from_secs(30);

use wantedspecies::*;

async fn download_img(client: &mut Client, img_url: &str, dst: &Path) -> Result<()> {
    let mut dst = File::create(dst).unwrap();
    let body = client
        .get(img_url)
        .send()
        .await?
        .body()
        .limit(BODY_SIZE)
        .await?;

    dst.write_all(&body).unwrap();

    Ok(())
}

async fn handle_record(
    client: &mut Client,
    record: csv::StringRecord,
    curated_data: &serde_yaml::Value,
) -> Result<Specie, std::io::Error> {
    let slug = slug::slugify(record.get(2).unwrap().to_string());
    let (img_credit_url, img_url) = match curated_data.get(&slug) {
        Some(s) => (
            s.get("img_credit_url")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            s.get("img_url").unwrap().as_str().unwrap().to_string(),
        ),
        _ => (String::from(""), String::from("")),
    };
    let mut specie = Specie {
        slug: slug,
        scientific_name: record.get(2).unwrap().to_string(),
        last_seen: record.get(19).unwrap().to_string(),
        category: record.get(3).unwrap().to_string(),
        img_credit_url: img_credit_url,
        assessment_id: record.get(0).unwrap().parse().unwrap(),
        internal_taxon_id: record.get(1).unwrap().parse().unwrap(),
        has_mugshot: false,
    };

    let mugshot_path = format!("static/mugshots/{}.jpg", specie.slug);
    let mugshot_path = Path::new(&mugshot_path);
    if mugshot_path.exists() {
        specie.has_mugshot = true;
    } else if !img_url.is_empty() && !mugshot_path.exists() {
        println!("Downloading {}", &specie.slug);
        match download_img(client, &img_url, &mugshot_path).await {
            Ok(_) => specie.has_mugshot = true,
            Err(e) => {
                specie.has_mugshot = false;
                remove_file(mugshot_path).unwrap();
                eprintln!("Error: {}", e);
            }
        };
    }
    Ok(specie)
}

#[actix_rt::main]
async fn main() {
    dotenv().ok();
    let curated_db_path = env::var("CURATED_DB").unwrap();
    let app_db_path = env::var("APP_DB").unwrap();
    let src_db_path = env::var("SRC_DB").unwrap();

    let connector = Connector::new().timeout(REQ_TIMEOUT).finish();
    let mut client = ClientBuilder::new()
        .connector(connector)
        .timeout(REQ_TIMEOUT)
        .finish();

    let re_last_seen = Regex::new(LAST_SEEN_REG).unwrap();
    let re_category = Regex::new(CATEGORY_REG).unwrap();

    let curated_db_file = File::open(curated_db_path).unwrap();
    let curated_data: serde_yaml::Value = serde_yaml::from_reader(curated_db_file).unwrap();
    let mut species = Species::new();

    let app_db_file = File::create(app_db_path).unwrap();
    let src_db_file = File::open(src_db_path).unwrap();

    let mut rdr = csv::Reader::from_reader(src_db_file);
    for result in rdr.records() {
        let record = result.unwrap();
        let specie = handle_record(&mut client, record, &curated_data)
            .await
            .unwrap();
        if re_last_seen.is_match(&specie.last_seen) && re_category.is_match(&specie.category) {
            species.insert(specie.slug.clone(), specie);
        }
        if species.len() >= MAX_DB_SIZE {
            break;
        }
    }
    serde_yaml::to_writer(app_db_file, &species).unwrap();
    println!("# {:?}", species.len());
}
