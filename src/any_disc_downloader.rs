use include_dir::*;
use lofty::config::*;
use lofty::file::*;
use lofty::ogg::*;
use serde_json::Value;
use serde_json::json;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use zip::ZipWriter;
use zip_extensions::deflate::zip_writer_extensions::ZipWriterExtensions;

#[allow(dead_code)]
static DECIMALS: f64 = 1000.0;

#[allow(dead_code)]
const IMAGE_DATA: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/image_data");
const TEMPLATE_DATA: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/pack_data");

#[allow(dead_code)]
fn write_to_json(path: String, json_val: &Value) -> Result<(), String> {
    let mut file = match File::create(AnyDiscDownloader::get_path_from_exe(Path::new(&path))) {
        Err(e) => return Err(format!("File Creation Error: {}", e).to_string()),
        Ok(f) => f,
    };
    let json_string = match serde_json::to_string_pretty(json_val) {
        Err(e) => return Err(format!("Json to String Error: {}", e).to_string()),
        Ok(s) => s,
    };
    if let Err(e) = file.write_all(json_string.as_bytes()) {
        return Err(format!("File Write Error{}", e).to_string());
    }
    Ok(())
}

#[allow(dead_code)]
fn create_jukebox_song_json(file_name: &String, name: &String, length: &f64) -> Result<(), String> {
    let mut juke_json = json!(
        {
            "comparator_output": 11,
            "description": "",
            "length_in_seconds": 0.0,
            "sound_event": {
                "sound_id": ""
            }
        }
    );
    juke_json["description"] = json!(name);
    juke_json["length_in_seconds"] = json!(length);
    juke_json["sound_event"]["sound_id"] = json!(format!("minecraft:music_disc.{}", file_name));
    write_to_json(
        format!(
            "download_data/any_disc_dp/data/any_disc/jukebox_song/{}.json",
            file_name
        ),
        &juke_json,
    )
}

#[allow(dead_code)]
fn create_items_json(disc_name: &String, file_name: &String) -> Result<(), String> {
    let mut items_json = json!(
        {
            "model" : {
                "type" : "minecraft:model",
                "model" : ""
            }
        }
    );
    items_json["model"]["model"] = json!(format!("any_disc:item/{}", disc_name));
    write_to_json(
        format!(
            "download_data/any_disc_rp/assets/any_disc/items/{}.json",
            file_name
        ),
        &items_json,
    )
}

#[allow(dead_code)]
fn create_models_json(disc_name: &String, file_name: &String) -> Result<(), String> {
    let mut items_json = json!(
        {
            "parent": "item/generated",
            "textures": {
                "layer0": "any_disc:item/beneath"
            }
        }
    );
    items_json["textures"]["layer0"] = json!(format!("any_disc:item/{}", disc_name));
    write_to_json(
        format!(
            "download_data/any_disc_rp/assets/any_disc/models/item/{}.json",
            file_name
        ),
        &items_json,
    )
}

#[allow(dead_code)]
fn get_ogg_duration(path: &str) -> Result<Duration, lofty::error::FileParseError> {
    // Probe the file from a path
    let mut file = File::open(AnyDiscDownloader::get_path_from_exe(Path::new(path)))?;

    // Extract properties (duration, bitrate, sample rate, etc.)
    let ogg_file = VorbisFile::read_from(&mut file, ParseOptions::new())?;

    Ok(ogg_file.properties().duration())
}
#[derive(Default)]
#[allow(dead_code)]
pub struct AnyDiscDownloader {}

#[allow(dead_code)]
impl AnyDiscDownloader {
    pub fn get_path_from_exe(dest: &Path) -> String {
        let exe_path = env::current_exe().expect("couldn't find executable source");
        let exe_parent = exe_path.parent().expect("couldn't find exe parent dir");
        let dest = Path::new(exe_parent).join(dest);
        dest.to_str().expect("couldn't get string path").to_string()
    }

    fn copy_template_data(dest: &Path) {
        TEMPLATE_DATA
            .extract(Path::new(&AnyDiscDownloader::get_path_from_exe(dest)))
            .expect("failed to collect template data");
    }
    pub fn copy_image_data(dest: &Path) {
        IMAGE_DATA
            .extract(Path::new(&AnyDiscDownloader::get_path_from_exe(dest)))
            .expect("failed to collect image data");
    }
    pub fn validate_file_name(name: &str) -> bool {
        // Check empty or too long (255 bytes max for most file systems)
        if name.is_empty() || name.len() > 255 {
            return false;
        }

        // Check for forbidden characters and control characters
        let forbidden = ['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>'];
        if name.contains(&forbidden[..]) || name.chars().any(|c| c.is_control()) {
            return false;
        }

        // Check for trailing spaces or dots
        if name.ends_with(' ') || name.ends_with('.') {
            return false;
        }

        // Check for Windows reserved names
        let upper_name = name.to_uppercase();
        let reserved = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];
        if reserved.contains(&upper_name.as_str()) {
            return false;
        }

        true
    }
    pub fn download(
        discs: serde_json::Value,
        songs: HashSet<PathBuf>,
        images: HashSet<PathBuf>,
    ) -> Result<(), String> {
        //delete download_data content
        let _ = fs::remove_dir_all(AnyDiscDownloader::get_path_from_exe(Path::new(
            "download_data",
        )));
        //copy template folder over
        AnyDiscDownloader::copy_template_data(Path::new(&AnyDiscDownloader::get_path_from_exe(
            Path::new("download_data"),
        )));
        //copy songs
        let song_folder_string = AnyDiscDownloader::get_path_from_exe(Path::new(
            "download_data/any_disc_rp/assets/minecraft/sounds/records",
        ));
        let song_folder = Path::new(&song_folder_string);
        for song in songs.iter() {
            let song_str = song
                .file_name()
                .expect("failed to convert song to string")
                .to_str()
                .expect("");
            let _ = fs::copy(song, song_folder.join(song_str));
        }
        let image_folder_string = AnyDiscDownloader::get_path_from_exe(Path::new(
            "download_data/any_disc_rp/assets/any_disc/textures/item",
        ));
        let image_folder = Path::new(&image_folder_string);
        for image in images.iter() {
            let image_str = image
                .file_name()
                .expect("failed to convert song to string")
                .to_str()
                .expect("");
            let _ = fs::copy(image, image_folder.join(image_str));
        }
        let mut creeper: Value = json!(
          {
        "type": "minecraft:entity",
        "pools": [
          {
            "bonus_rolls": 0,
            "condition":
              {
                "type": "minecraft:entity_properties",
                "entity": "attacker",
                "predicate":{
                  "equipment": {
                    "offhand": {
                      "items": [
                        "minecraft:breeze_rod"
                      ],
                      "components": {
                        "minecraft:enchantments": {
                          "any_disc:mark_of_music": 1
                        }
                      }
                    }
                  }
                }
              }
            ,
            "entries": [],
            "rolls": 1
          },
          {
            "bonus_rolls": 0,
            "entries": [
              {
                "type": "minecraft:item",
                "modifier": [
                  {
                    "count": {
                      "type": "minecraft:uniform",
                      "max": 2.0,
                      "min": 0.0
                    },
                    "type": "minecraft:set_count"
                  },
                  {
                    "count": {
                      "type": "minecraft:uniform",
                      "max": 1.0,
                      "min": 0.0
                    },
                    "enchantment": "minecraft:looting",
                    "type": "minecraft:enchanted_count_increase"
                  }
                ],
                "name": "minecraft:gunpowder"
              }
            ],
            "rolls": 1
          },
          {
            "bonus_rolls": 0,
            "condition":
              {
                "type": "minecraft:entity_properties",
                "entity": "attacker",
                "predicate": {
                  "minecraft:entity_type": "#minecraft:skeletons"
                }
              }
            ,
            "entries": [
              {
                "type": "minecraft:tag",
                "expand": true,
                "items": "#minecraft:creeper_drop_music_discs"
              }
            ],
            "rolls": 1
          }
        ],
        "random_sequence": "minecraft:entities/creeper"
          }
          );
        let creeper_vec = creeper["pools"][0]["entries"].as_array_mut().unwrap();

        let mut music_disc_11 = json!(
        {
        "model": {
            "type": "minecraft:range_dispatch",
            "property": "minecraft:custom_model_data",
            "index": 0,
            "fallback": {
                "type": "minecraft:model",
                "model": "minecraft:item/music_disc_11"
            },
            "entries": []
        }
        });
        let music_disc_11_vec = music_disc_11["model"]["entries"].as_array_mut().unwrap();
        let mut sounds = json!({});
        let sounds_obj = sounds.as_object_mut().unwrap();

        let disc_array = match discs["list"].as_array() {
            Some(discs) => discs,
            None => return Err("Discs invalid Format".to_string()),
        };

        //First look to identify weights of albums
        let mut max_weight = 100.0;
        let mut albums = json!({});

        for disc in disc_array {
            //Only check albums for sizes
            if disc.get("album").is_some() && disc.get("weight").is_none() {
                //First occurence of album check
                if albums.get(disc["album"].as_str().unwrap()).is_none() {
                    albums[disc["album"].as_str().unwrap()] = json!(0.0);
                }

                //Update size and compare to maximum size
                albums[disc["album"].as_str().unwrap()] =
                    json!(albums[disc["album"].as_str().unwrap()].as_f64().unwrap() + 1.0);
                if albums[disc["album"].as_str().unwrap()].as_f64().unwrap() > max_weight {
                    max_weight = albums[disc["album"].as_str().unwrap()].as_f64().unwrap();
                }
            }
        }

        //Now update each disc
        for (i, disc) in disc_array.iter().enumerate() {
            let file_name = disc["file_name"].clone().as_str().unwrap().to_string();
            let name = disc["name"].clone().as_str().unwrap().to_string();
            let path = format!(
                "download_data/any_disc_rp/assets/minecraft/sounds/records/{}.ogg",
                file_name
            );
            let length = get_ogg_duration(path.as_str()).unwrap().as_secs_f64();
            //default weight for independent song
            let mut weight = max_weight;
            //Create Song

            if let Err(s) = create_jukebox_song_json(&file_name, &name, &length) {
                return Err(format!(
                    "Failed to create jukebox_song_json ({}), error: {}",
                    file_name, s
                ));
            }

            if disc.get("album").is_some() && disc["album"].as_str().unwrap() != "" {
                //Create disc in album
                if discs["albums"]
                    .get(disc["album"].as_str().unwrap())
                    .is_some()
                {
                    let disc_file = discs["albums"][disc["album"].as_str().unwrap()]["disc_file"]
                        .as_str()
                        .unwrap()
                        .to_string();
                    if let Err(s) = create_items_json(&disc_file, &file_name) {
                        return Err(format!(
                            "Failed to create items_json ({}), error: {}",
                            file_name, s
                        ));
                    }
                    if let Err(s) = create_models_json(&disc_file, &file_name) {
                        return Err(format!(
                            "Failed to create items_json ({}), error: {}",
                            file_name, s
                        ));
                    }
                    //only change weights for albums (independent songs should have the max weight)
                    weight = ((max_weight
                        / albums[disc["album"].as_str().unwrap()].as_f64().unwrap())
                        * DECIMALS)
                        .trunc()
                        / DECIMALS;
                } else {
                    return Err(format!(
                        "ERROR: album {} for disc {} not a valid album",
                        disc["album"], disc["file_name"]
                    ));
                }
            } else {
                //Create independent disc
                if let Err(s) = create_items_json(&file_name, &file_name) {
                    return Err(format!(
                        "Failed to create items_json ({}), error: {}",
                        file_name, s
                    ));
                }
                if let Err(s) = create_models_json(&file_name, &file_name) {
                    return Err(format!(
                        "Failed to create items_json ({}), error: {}",
                        file_name, s
                    ));
                }
            }

            //check weight override
            if disc.get("weight").is_some() && disc["weight"].is_f64() {
                let test_weight = disc["weight"].as_f64().unwrap();
                if test_weight > 0.0 {
                    weight = disc["weight"].as_f64().unwrap();
                }
            }

            //update creeper loot elements
            creeper_vec.push(json!(
            {
              "type": "minecraft:item",
              "weight": json!(weight),
              "name": "minecraft:music_disc_11",
              "modifier": [
                {
                  "type": "minecraft:set_components",
                  "components": {
                    "minecraft:custom_model_data": {
                      "floats": [json!(i+1)]
                    },
                    "minecraft:jukebox_playable": json!(format!("any_disc:{}", file_name))
                  }
                }
              ]
            }
            ));

            //update music disc 11 models
            music_disc_11_vec.push(json!(
                {
                    "model": {
                        "type": "minecraft:model",
                        "model": json!(format!("any_disc:item/{}", file_name))
                    },
                    "threshold" : json!(i+1)
                }
            ));

            //insert new sounds object
            sounds_obj.insert(
                format!("music_disc.{}", file_name),
                json!(
                {
                    "sounds": [
                        {
                            "name": json!(format!("records/{}", file_name)),
                            "stream": true,
                            "attenuation_distance": 6
                        }
                    ],
                }
                ),
            );
        }

        //Update jsons
        creeper["pools"][0]["entries"] = json!(creeper_vec);
        music_disc_11["model"]["entries"] = json!(music_disc_11_vec);
        sounds = json!(sounds_obj);

        //write creeper.json, music_disc_11.json, and sounds.json
        if let Err(s) = write_to_json(
            "download_data/any_disc_dp/data/minecraft/loot_table/entities/creeper.json".to_string(),
            &creeper,
        ) {
            return Err(format!("Failed to create creeper.json, error: {}", s));
        }
        if let Err(s) = write_to_json(
            "download_data/any_disc_rp/assets/minecraft/items/music_disc_11.json".to_string(),
            &music_disc_11,
        ) {
            return Err(format!("Failed to create music_disc_11.json, error: {}", s));
        }
        if let Err(s) = write_to_json(
            "download_data/any_disc_rp/assets/minecraft/sounds.json".to_string(),
            &sounds,
        ) {
            return Err(format!("Failed to create sounds.json, error: {}", s));
        }
        if let Err(s) = write_to_json("download_data/discs.json".to_string(), &discs) {
            return Err(format!("Failed to create discs.json, error: {}", s));
        }
        let source_dir = PathBuf::from(AnyDiscDownloader::get_path_from_exe(Path::new(
            "download_data",
        )));
        let target_zip = PathBuf::from(AnyDiscDownloader::get_path_from_exe(Path::new(
            "any_disc.zip",
        )));

        // 2. Create the target physical file
        let file = match File::create(&target_zip) {
            Ok(f) => f,
            Err(e) => return Err(format!("Failed to create initial empty zip: {}", e)),
        };
        // 3. Pass it to the standard ZipWriter
        let mut zip = ZipWriter::new(file);
        // 4. Call the trait method to compress the whole directory hierarchy
        if let Err(e) = zip.create_from_directory(&source_dir) {
            return Err(format!("Failed to make zip: {}", e));
        }

        //delete excess
        let _ = fs::remove_dir_all(AnyDiscDownloader::get_path_from_exe(Path::new(
            "download_data",
        )));
        Ok(())
    }
}
