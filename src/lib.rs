use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::path::Path;

const MUGSHOT_PATH: &str = "static/mugshots";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Specie {
    pub slug: String,
    pub scientific_name: String,
    pub last_seen: String,
    pub category: String,
    pub img_credit_url: String,
    pub assessment_id: u64,
    pub internal_taxon_id: u64,
    pub has_mugshot: bool,
}

pub type Species = HashMap<String, Specie>;

impl Specie {
    pub fn get_iucn_url(&self) -> String {
        format!(
            "https://www.iucnredlist.org/species/{}/{}",
            self.internal_taxon_id, self.assessment_id
        )
    }

    pub fn get_permalink(&self) -> String {
        format!("/{}", self.slug)
    }

    pub fn get_mugshot_url(&self) -> String {
        if self.has_mugshot {
            format!("/{}/{}.jpg", MUGSHOT_PATH, self.slug)
        } else {
            String::from("/static/no-mugshot.svg")
        }
    }
}

/// Forget the items that have no image in the database
pub fn forget_no_image(species: &mut Species) {
    let mut to_remove = vec![];
    for (k, v) in species.iter() {
        let filename = format!("{}.jpg", &v.slug);
        if !Path::new(MUGSHOT_PATH).join(filename).exists() {
            to_remove.push(k.to_owned());
        }
    }

    for k in to_remove {
        species.remove(&k);
    }
    println!("Db size: {}", species.len());
}

pub fn tag_no_image(species: &mut Species) {
    for (_, v) in species.iter_mut() {
        let filename = format!("{}.jpg", &v.slug);
        v.has_mugshot = Path::new(MUGSHOT_PATH).join(filename).exists();
    }
}
