// #![windows_subsystem = "windows"]
mod album;
mod any_disc_downloader;
mod disc;

use album::*;
use any_disc_downloader::*;
use disc::*;
use eframe::egui;
use serde_json::*;
use std::path::*;
use std::{collections::HashSet, fs};

static DISC_SONG_INDENT: f32 = 9.0;
static DISC_ALBUM_INDENT: f32 = 0.0;
static DISC_NAME_INDENT: f32 = 4.0;
static HEADER_GAP: f32 = 8.0;
static ELEMENT_GAP: f32 = 5.0;
static ALBUM_NAME_INDENT: f32 = 1.0;
static ALBUM_IMAGE_INDENT: f32 = 0.0;
// static IMAGE_COLUMN_PERCENTAGE: f32 = 0.10;
// static SONG_COLUMN_PERCENTAGE: f32 = 0.10;
// static CREATION_COLUMN_PERCENTAGE: f32 = 0.10;
// static ALBUM_COLUMN_PERCENTAGE: f32 = 0.35;
// static DISC_COLUMN_PERCENTAGE: f32 = 0.35;

fn load_icon() -> egui::viewport::IconData {
    // Load image using the `image` crate (or decode raw bytes)
    let image = image::open(AnyDiscDownloader::get_path_from_exe(Path::new(
        "image_data/pack.png",
    )))
    .expect("Failed to open icon path")
    .to_rgba8();
    let (width, height) = image.dimensions();

    egui::viewport::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
fn main() {
    let _ = fs::remove_dir_all(AnyDiscDownloader::get_path_from_exe(Path::new(
        "image_data",
    )));
    //copy template folder over
    let _ = std::fs::create_dir_all(AnyDiscDownloader::get_path_from_exe(Path::new(
        "image_data",
    )));
    AnyDiscDownloader::copy_image_data(Path::new(&AnyDiscDownloader::get_path_from_exe(
        Path::new("image_data"),
    )));
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_icon(load_icon()), // Set your custom icon here
        ..Default::default()
    };
    let _ = eframe::run_native(
        "AnyDisc",
        native_options,
        Box::new(|cc| Ok(Box::new(AnyDiscApp::new(cc)))),
    );
}

#[derive(Default)]
struct AnyDiscApp {
    discs: Vec<Disc>,
    albums: Vec<Album>,
    ogg_paths: HashSet<PathBuf>,
    png_paths: HashSet<PathBuf>,
    ogg_file_names: HashSet<String>,
    png_file_names: HashSet<String>,
    upload_status: i8,
}

impl AnyDiscApp {
    fn default() -> Self {
        Self {
            upload_status: 1,
            albums: vec![Album::new("default".to_string(), "default".to_string())],
            png_paths: [PathBuf::from(AnyDiscDownloader::get_path_from_exe(
                Path::new("image_data/default.png"),
            ))]
            .into_iter()
            .collect(),
            png_file_names: ["default.png".to_string()].into_iter().collect(),
            ..Default::default()
        }
    }
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    fn select_files(extension: &str, title: &str) -> Vec<PathBuf> {
        let path = rfd::FileDialog::new()
            .set_title(title)
            .add_filter(extension, &[extension])
            .pick_files();
        if let Some(path) = path {
            return path;
        }
        vec![]
    }
    fn select_file_name(extension: &str, title: &str) -> (Option<PathBuf>, String) {
        // create path
        let path = rfd::FileDialog::new()
            .set_title(title)
            .add_filter(extension, &[extension])
            .pick_file();
        let string: String;
        match path {
            Some(p) => {
                string = p.file_name().unwrap().to_str().unwrap().to_string();
                (Some(p), string[0..string.rfind(".").unwrap()].to_string())
            }
            None => (None, "".to_string()),
        }
    }
    fn get_discs_import_data(&mut self) {
        // create path
        let path = rfd::FileDialog::new()
            .set_title("select a json")
            .add_filter("json", &["json"])
            .pick_file();
        // create string and set json
        let disc_string: String;
        if let Some(path) = path {
            disc_string = fs::read_to_string(path).expect("Failed to read file selected");
        } else {
            return;
        }
        self.discs.clear();
        self.albums.clear();
        let json: Value =
            serde_json::from_str(disc_string.as_str()).expect("Couldn't convert json to string");
        // validate json (Basic)
        if json.get("list").is_some() && json.get("albums").is_some() {
            let discs_array = json["list"].as_array().unwrap();
            let albums = json["albums"].as_object().unwrap();
            for disc in discs_array {
                self.discs.push(Disc::new(
                    disc["name"].as_str().unwrap_or("").to_string(),
                    disc["file_name"].as_str().unwrap_or("").to_string(),
                    disc["album"].as_str().unwrap_or("").to_string(),
                ));
            }
            for (key, val) in albums.iter() {
                self.albums.push(Album::new(
                    key.to_string(),
                    val["disc_file"].as_str().unwrap().to_string(),
                ));
            }
        }
    }
    fn validate(&mut self) -> bool {
        if self.ogg_file_names.len() != self.ogg_paths.len() {
            return false;
        }
        if self.png_file_names.len() != self.png_paths.len() {
            return false;
        }
        for song in self.ogg_paths.iter() {
            if !song.exists() {
                return false;
            }
        }
        for image in self.png_paths.iter() {
            if !image.exists() {
                return false;
            }
        }
        for disc in self.discs.iter() {
            let needed_file_name = disc.ogg_name.clone() + ".ogg";
            if !self.ogg_file_names.contains(&needed_file_name) {
                return false;
            }
            if disc.album.is_empty()
                && !self
                    .png_file_names
                    .contains(&(disc.ogg_name.clone() + ".png"))
            {
                return false;
            }
        }
        for album in self.albums.iter() {
            let needed_file_name = album.png_name.clone() + ".png";
            if !self.png_file_names.contains(&needed_file_name) {
                return false;
            }
        }
        true
    }

    fn upload_data(&mut self) {
        if !self.validate() {
            self.upload_status = 0;
            return;
        }
        //make discs.json complete
        let mut discs_json = json!({
            "list" : [],
            "albums" : {

            }
        });
        let discs_array = discs_json["list"].as_array_mut().unwrap();
        for disc in self.discs.iter() {
            if disc.view_name.is_empty() || disc.ogg_name.is_empty() || disc.ogg_name.contains(".")
            {
                self.upload_status = 0;
                return;
            }
            discs_array.push(json!(
                {
                    "name": disc.view_name,
                    "file_name": disc.ogg_name,
                    "album": disc.album
                }
            ));
        }
        let albums = discs_json.get_mut("albums").unwrap();
        for album in self.albums.iter() {
            if album.name.is_empty() || album.png_name.is_empty() || album.png_name.contains(".") {
                self.upload_status = 0;
                return;
            }
            albums[album.name.clone()] = json!(
                {
                    "disc_file": album.png_name
            });
        }
        let download = AnyDiscDownloader::download(
            discs_json.clone(),
            self.ogg_paths.clone(),
            self.png_paths.clone(),
        );
        match download {
            Ok(_) => self.upload_status = 2,
            Err(_) => {
                self.upload_status = 0;
            }
        }
    }
}

impl eframe::App for AnyDiscApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("AnyDisc");
            ui.add_space(HEADER_GAP);
            ui.columns(5, |columns| {
                //DISC COLUMN
                columns[3].push_id(7, |disc_header_ui| {
                    egui::ScrollArea::horizontal().show(disc_header_ui, |disc_header_ui| {
                        disc_header_ui.horizontal(|disc_header_ui| {
                            disc_header_ui.heading("Discs");
                            if disc_header_ui.button("Add New Disc").clicked() {
                                self.discs.push(Disc::new(
                                    "".to_string(),
                                    "".to_string(),
                                    "default".to_string(),
                                ));
                            }
                            if disc_header_ui.button("Import discs.json").clicked() {
                                self.get_discs_import_data();
                            }
                        });
                        disc_header_ui.add_space(HEADER_GAP);
                    });
                });
                columns[3].push_id(3, |discs_ui| {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(
                        discs_ui,
                        |discs_ui| {
                            for (i, disc) in self.discs.iter_mut().enumerate() {
                                discs_ui.horizontal(|disc_ui| {
                                    disc_ui.label(format!("Disc {}:", i));
                                    disc_ui.vertical(|disc_ui| {
                                        disc_ui.horizontal(|disc_ui| {
                                            disc_ui.label("Name");
                                            disc_ui.add_space(DISC_NAME_INDENT);
                                            disc_ui.text_edit_singleline(&mut disc.view_name);
                                        });
                                        disc_ui.horizontal(|disc_ui| {
                                            disc_ui.label("Song".to_string());
                                            disc_ui.add_space(DISC_SONG_INDENT);
                                            let button_name = match disc.ogg_name.is_empty() {
                                                true => format!("Select .ogg File for Disc {}", i),
                                                false => format!("{}.ogg", disc.ogg_name)
                                            };
                                            if disc_ui
                                                .button(button_name)
                                                .clicked()
                                            {
                                                let (path, name) = AnyDiscApp::select_file_name(
                                                    "ogg",
                                                    format!("Select .ogg For Disc {}", i).as_str(),
                                                );
                                                if let Some(p) = path {
                                                    let p_file_name = p.file_name().unwrap().to_str().unwrap().to_string();
                                                    self.ogg_paths.insert(p);
                                                    self.ogg_file_names.insert(p_file_name);
                                                }
                                                disc.ogg_name = name;
                                            }
                                            let has_album = !disc.album.is_empty();
                                            let has_ogg = self.ogg_file_names.contains(&(disc.ogg_name.clone() + ".ogg"));
                                            let has_png = self.png_file_names.contains(&(disc.ogg_name.clone() + ".png"));
                                            if !has_ogg{
                                                if has_album || has_png {
                                                    disc_ui.label(egui::RichText::new("* Can't Find Song!").color(egui::Color32::RED));
                                                }else{
                                                    disc_ui.label(egui::RichText::new("* Can't Find Song or Image!").color(egui::Color32::RED));
                                                }
                                            }else{
                                                if !has_album && !has_png{
                                                    disc_ui.label(egui::RichText::new("* Can't Find Image!").color(egui::Color32::RED));                                                }
                                            }
                                        });
                                        disc_ui.horizontal(|disc_ui| {
                                            disc_ui.label("Album");
                                            disc_ui.add_space(DISC_ALBUM_INDENT);
                                            egui::ComboBox::new(i, "Choose an Album")
                                            .selected_text(disc.album.to_string())
                                            .show_ui(disc_ui, |disc_ui|{
                                                disc_ui.selectable_value(&mut disc.album, "".to_string(), "");
                                                for album in self.albums.iter(){
                                                    disc_ui.selectable_value(&mut disc.album, album.name.clone(), album.name.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                                discs_ui.add_space(ELEMENT_GAP);
                            }
                        },
                    );
                });

                //ALBUM COLUMN
                columns[2].push_id(6, |album_header_ui| {
                    egui::ScrollArea::horizontal().show(album_header_ui, |album_header_ui| {
                        album_header_ui.horizontal(|album_header_ui| {
                            album_header_ui.heading("Albums");
                            if album_header_ui.button("Add New Album").clicked() {
                                self.albums.push(Album::new("".to_string(), "".to_string()));
                            }
                        });
                        album_header_ui.add_space(HEADER_GAP);
                    });
                });
                columns[2].push_id(2, |albums_ui| {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(
                        albums_ui,
                        |albums_ui| {
                            for (i, album) in self.albums.iter_mut().enumerate() {
                                albums_ui.horizontal(|album_ui| {
                                    album_ui.label(format!("Album {}:", i));
                                    album_ui.vertical(|album_ui| {
                                        album_ui.horizontal(|album_ui| {
                                            album_ui.label("Name");
                                            album_ui.add_space(ALBUM_NAME_INDENT);
                                            album_ui.text_edit_singleline(&mut album.name);
                                        });
                                        album_ui.horizontal(|album_ui| {
                                            album_ui.label("Image".to_string());
                                            album_ui.add_space(ALBUM_IMAGE_INDENT);
                                            let button_name = match album.png_name.is_empty() {
                                                true => format!("Select .png File for Album {}", i),
                                                false => format!("{}.png", album.png_name)
                                            };
                                            if album_ui
                                                .button(button_name)
                                                .clicked()
                                            {
                                                let (path, name) = AnyDiscApp::select_file_name(
                                                    "png",
                                                    format!("Select .png For Album {}", i).as_str(),
                                                );
                                                if let Some(p) = path {
                                                    let p_file_name = p.file_name().unwrap().to_str().unwrap().to_string();
                                                    self.png_paths.insert(p);
                                                    self.png_file_names.insert(p_file_name);
                                                }
                                                album.png_name = name;
                                            }
                                            let needed_file_name = album.png_name.clone() + ".png";
                                            if !self.png_file_names.contains(&needed_file_name) {
                                                album_ui.label(egui::RichText::new("* Can't Find Image!").color(egui::Color32::RED));
                                            }
                                        });
                                    });
                                });
                                albums_ui.add_space(ELEMENT_GAP);
                            }
                        },
                    );
                });
                //SONGS COLUMN
                columns[1].push_id(5, |songs_header_ui| {
                    egui::ScrollArea::horizontal().show(songs_header_ui, |songs_header_ui| {
                        songs_header_ui.horizontal(|songs_header_ui| {
                            songs_header_ui.heading("Songs");
                            if songs_header_ui.button("Add Songs").clicked() {
                                let selected_oggs =
                                    AnyDiscApp::select_files("ogg", "Select .ogg Files");
                                for ogg in selected_oggs {
                                    let ogg_file_name = ogg.file_name().unwrap().to_str().unwrap().to_string();
                                    self.ogg_paths.insert(ogg);
                                    self.ogg_file_names.insert(ogg_file_name);
                                }
                            }
                            if self.ogg_file_names.len() != self.ogg_paths.len(){
                                songs_header_ui.label(egui::RichText::new("* Duplicates Found").color(egui::Color32::RED));
                            }
                        });
                        songs_header_ui.add_space(HEADER_GAP);
                    });
                });
                columns[1].push_id(1, |songs_ui| {
                    let mut removed_songs: HashSet<&PathBuf> = Default::default();
                    let ogg_paths_copy = self.ogg_paths.clone();
                    egui::ScrollArea::both().auto_shrink([false, false]).show(
                        songs_ui,
                        |songs_ui| {
                            let path_vec = ogg_paths_copy.iter().map(|i| i.file_name().unwrap().to_str().unwrap());
                            for song in ogg_paths_copy.iter() {
                                let song_str =
                                    song.file_name().expect("could not move path to string").to_str().unwrap();
                                songs_ui.horizontal(|song_ui|{
                                    let duplicate = path_vec.clone().filter(|i| *i == song_str).count() > 1;
                                    let color = match duplicate {
                                        false => egui::Color32::PLACEHOLDER,
                                        true => egui::Color32::RED
                                    };
                                    song_ui.add(
                                    egui::Label::new(egui::RichText::new(
                                        song_str.to_string()
                                    ).color(color)
                                    )
                                    .extend(),
                                    );
                                    if song_ui.button("Remove").clicked() {
                                        if !duplicate{
                                            self.ogg_file_names.remove(song_str);
                                        }
                                        removed_songs.insert(song);
                                    }
                                    if !song.exists(){
                                        song_ui.label(egui::RichText::new("* Can't find File!").color(egui::Color32::RED));
                                    }
                                });
                                songs_ui
                                    .label(egui::RichText::new(format!("[{}]", song.to_str().unwrap())).small());
                                songs_ui.add_space(ELEMENT_GAP);
                            }
                        },
                    );
                    self.ogg_paths.retain(|i| {!removed_songs.contains(i)});
                });

                //IMAGES COLUMN
                columns[0].push_id(4, |images_header_ui| {
                    egui::ScrollArea::horizontal().show(images_header_ui, |images_header_ui| {
                        images_header_ui.horizontal(|images_header_ui| {
                            images_header_ui.heading("Images");
                            if images_header_ui.button("Add Images").clicked() {
                                let selected_pngs =
                                    AnyDiscApp::select_files("png", "Select .png Files");
                                for png in selected_pngs {
                                    let png_file_name = png.file_name().unwrap().to_str().unwrap().to_string();
                                    self.png_paths.insert(png);
                                    self.png_file_names.insert(png_file_name);
                                }
                            }
                            if self.png_file_names.len() != self.png_paths.len(){
                                images_header_ui.label(egui::RichText::new("* Duplicates Found").color(egui::Color32::RED));
                            }
                        });
                        images_header_ui.add_space(HEADER_GAP);
                    });
                });
                columns[0].push_id(0, |images_ui| {
                    let mut removed_images: HashSet<&PathBuf> = Default::default();
                    let png_paths_copy = self.png_paths.clone();
                    {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(
                        images_ui,
                        |images_ui| {
                            let path_vec = png_paths_copy.iter().map(|i| i.file_name().unwrap().to_str().unwrap());
                            for image in png_paths_copy.iter() {
                                let image_str =
                                    image.file_name().expect("could not move path to string").to_str().unwrap();
                                images_ui.horizontal(|image_ui|{
                                    let duplicate = path_vec.clone().filter(|i| *i == image_str).count() > 1;
                                    let color = match duplicate{
                                        false => egui::Color32::PLACEHOLDER,
                                        true => egui::Color32::RED
                                    };
                                    image_ui.add(
                                    egui::Label::new(egui::RichText::new(
                                        image_str
                                            .to_string()
                                        ).color(color))
                                        .extend(),
                                    );
                                    if image_ui.button("Remove").clicked() {
                                        if !duplicate{
                                            self.png_file_names.remove(image_str);
                                        }
                                        removed_images.insert(image);
                                    }
                                    if !image.exists(){
                                        image_ui.label(egui::RichText::new("* Can't find File!").color(egui::Color32::RED));
                                    }
                                });
                                images_ui
                                    .label(egui::RichText::new(format!("[{}]", image.to_str().unwrap())).small());
                                images_ui.add_space(ELEMENT_GAP);
                            }
                        },
                    );
                    }
                    self.png_paths.retain(|i| {!removed_images.contains(i)});
                });

                //CREATION COLUMN
                columns[4].push_id(8, |create_ui| {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(
                        create_ui,
                        |create_ui| {
                            create_ui.label(
                                "*discs with empty albums require a .png file of the same name as their .ogg file",
                            );
                            create_ui.add_space(10.0);
                            create_ui.horizontal(|create_ui| {
                                if create_ui.button("Create Packs").clicked() {
                                    self.upload_data();
                                }
                                if self.upload_status == 0 {
                                    create_ui.label("Status: Creation Failed");
                                }else if self.upload_status == 1 {
                                    create_ui.label("Status: Ready");
                                }else{
                                    create_ui.label("Status: Creation Successful");
                                }
                            });
                        },
                    );
                });
            });
        });
    }
}
