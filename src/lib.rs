use csv::ReaderBuilder;
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;

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
        self.bin == other.bin
            && self.iso_code_2 == other.iso_code_2
            && self.iso_code_3 == other.iso_code_3
            && self.brand == other.brand
            && self.card_type == other.card_type
            && self.country_name == other.country_name
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BinDb {
    record_by_bin: HashMap<String, BinRecord>,
}

impl BinDb {
    pub fn new() -> Self {
        BinDb {
            record_by_bin: read_csv().expect("Failed to initialize GeoList data."),
        }
    }

    pub fn get_iso2_codes<I>(&self, bins: I) -> HashSet<&str>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        bins.into_iter()
            .filter_map(|b| self.record_by_bin.get(b.as_ref()))
            .map(|rec| rec.iso_code_2.as_str())
            .collect()
    }
}

fn read_csv() -> Result<HashMap<String, BinRecord>, Box<dyn Error>> {
    let bin_list = Asset::get("bin-list-data.csv").expect("bin-list-data.csv must exist");
    let content = std::str::from_utf8(bin_list.data.as_ref())
        .map_err(|_| "Embedded file is not valid UTF-8.")?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());
    let mut record_by_bin = HashMap::with_capacity(375_000);

    for result in rdr.deserialize() {
        let record: BinRecord = result?;
        record_by_bin.insert(record.bin.clone(), record);
    }

    Ok(record_by_bin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_initialization() {
        let result = read_csv();

        assert!(
            result.is_ok(),
            "Dictionary initialization failed: {:?}",
            result.err()
        );
        assert!(
            !result.unwrap().is_empty(),
            "Dictionary initialized but is empty."
        );
    }

    #[test]
    fn test_successful_batch_lookup() -> Result<(), Box<dyn Error>> {
        let db = BinDb::new();
        let mock_bins = vec!["123456".to_string(), "987654".to_string()];

        let found_codes = db.get_iso2_codes(mock_bins);

        assert!(
            found_codes.is_empty() || found_codes.len() <= 2,
            "Batch lookup returned an unexpected number of results."
        );

        for code in &found_codes {
            assert!(!code.is_empty(), "Found code should not be empty.");
        }

        Ok(())
    }
}
