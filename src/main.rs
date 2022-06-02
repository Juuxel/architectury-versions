use std::io::Write;
use std::str::FromStr;
use architectury_versions::{architectury::*, version::*};
use comfy_table::{Cell, Color, Table};
use roxmltree::Document;

const ARCHITECTURY_JSON_URL: &str = "https://gist.githubusercontent.com/shedaniel/4a37f350a6e49545347cb798dbfa72b3/raw/architectury.json";

fn main() -> reqwest::Result<()> {
    let version = std::env::args().skip(1).next();

    if let Some(v) = &version {
        eprint!("Fetching data for {}...", v);
    } else {
        eprint!("Fetching data for latest version...");
    }
    std::io::stderr().flush().unwrap();

    let data = get_architectury_data()?.unwrap();
    let version_data = version.and_then(|v| data.versions.get(&v)).unwrap_or_else(|| data.stable());

    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .apply_modifier(comfy_table::modifiers::UTF8_SOLID_INNER_BORDERS)
        .add_row(vec![
            Cell::new("Architectury Loom").fg(Color::Green),
            Cell::new("Architectury Plugin").fg(Color::Green)
        ])
        .add_row(vec![
            get_latest(data.resolve(&version_data.loom).unwrap())?.unwrap(),
            get_latest(data.resolve(&version_data.plugin).unwrap())?.unwrap(),
        ])
        .add_row(vec![
            Cell::new("Architectury API").fg(Color::Green),
            Cell::new("Injectables").fg(Color::Green)
        ])
        .add_row(vec![
            get_latest(data.resolve(&version_data.api).unwrap())?.unwrap(),
            get_latest(data.resolve(&version_data.injectables).unwrap())?.unwrap(),
        ]);

    print!("\r");
    println!("{table}");
    Ok(())
}

fn get_architectury_data() -> reqwest::Result<Option<ArchitecturyJson>> {
    let response = reqwest::blocking::get(ARCHITECTURY_JSON_URL)?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let text = response.text()?;
    let json: serde_json::Value = serde_json::from_str(text.as_str()).unwrap();
    Ok(ArchitecturyJson::from_json(&json))
}

fn get_latest(def: &VersionDefinition) -> reqwest::Result<Option<String>> {
    let versions = get_versions(def.pom.as_str())?;
    if versions.is_none() {
        return Ok(None);
    }
    Ok(
        versions.unwrap().iter()
            .filter(|s| def.filter.is_match(s))
            .map(|s| Version::from_str(s).unwrap())
            .max()
            .map(|v| v.to_string())
    )
}

fn get_versions(maven_metadata_url: &str) -> reqwest::Result<Option<Vec<String>>> {
    let response = reqwest::blocking::get(maven_metadata_url)?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let text = response.text()?;
    let document = Document::parse(text.as_str()).unwrap();
    let metadata_node = document.root_element();
    let mut result: Vec<Option<String>> = Vec::new();

    for child in metadata_node.children() {
        if child.has_tag_name("versioning") {
            for child in child.children() {
                if child.has_tag_name("versions") {
                    for child in child.children() {
                        result.push(child.text().map(|s| String::from(s)))
                    }
                }
            }
        }
    }

    Ok(result.into_iter().collect())
}
