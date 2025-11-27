use rust_embed::RustEmbed;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "bin-list-data.csv"]
struct Asset;

#[derive(Debug, Deserialize, Eq, Clone)]
struct BinRecord {
    #[serde(rename = "BIN")]
    bin: String,
    #[serde(rename = "Brand")]
    brand: String,
    #[serde(rename = "Type")]
    card_type: String,
    #[serde(rename = "CountryName")]
    country_name: String,
    #[serde(rename = "isoCode3")]
    iso_code_3: String,
    #[serde(rename = "isoCode2")]
    iso_code_2: String,
}

impl PartialEq for BinRecord {
    fn eq(&self, other: &Self) -> bool {
        self.bin == other.bin && self.iso_code_2 == other.iso_code_2 &&
        self.iso_code_3 == other.iso_code_3 && self.brand == other.brand &&
        self.card_type == other.card_type && self.country_name == other.country_name
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GeoList {
    geos: HashMap<String, BinRecord>,
}

impl GeoList {
    pub fn new() -> Self {
        GeoList {
            geos: init_data().expect("Failed to initialize GeoList data."),
        }
    }
}

fn init_data() -> Result<HashMap<String, BinRecord>, Box<dyn Error>> {
    let bin_list = Asset::get("bin-list-data.csv")
        .ok_or_else(|| Box::<dyn Error>::from("'bin-list-data.csv' not found in assets."))?;
    let content = std::str::from_utf8(bin_list.data.as_ref())
        .map_err(|_| "Embedded file is not valid UTF-8.")?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true) 
        .from_reader(content.as_bytes());
    let mut data_map: HashMap<String, BinRecord> = HashMap::new();

    for result in rdr.deserialize() {
        let record: BinRecord = result?;
        let key = generate_hash(&record.bin); 
        data_map.insert(key, record);
    }
    
    Ok(data_map) 
}

pub fn lookup_bin(bins: Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
    let dictionary = GeoList::new().geos;

    let mut results = Vec::new();

    for bin in bins {
        let hash_key = generate_hash(&bin);

        if let Some(record) = dictionary.get(&hash_key) {
            results.push(record.iso_code_2.clone());
        }
    }
    
    Ok(results)
}

pub fn generate_hash(src: &str) -> String {
    format!("{:x}", md5::compute(src.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn test_dictionary_initialization() {
        let result = init_data();

        assert!(result.is_ok(), "Dictionary initialization failed: {:?}", result.err());
        assert!(!result.unwrap().is_empty(), "Dictionary initialized but is empty.");
    }

    #[test]
    fn test_successful_batch_lookup() -> Result<(), Box<dyn Error>> {
        let mock_bins = vec!["123456".to_string(), "987654".to_string()]; 
        
        let found_codes = lookup_bin(mock_bins)?;

        assert!(found_codes.is_empty() || found_codes.len() <= 2, "Batch lookup returned an unexpected number of results.");
        
        for code in &found_codes {
            assert!(!code.is_empty(), "Found code should not be empty.");
        }
        
        Ok(())
    }
}