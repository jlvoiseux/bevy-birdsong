use bevy::{prelude::*, text::Text2dBounds};
use std::collections::HashMap;
use std::time::Duration;
use crate::components::{DialogueBoxComponent, VoiceComponent, PortraitComponent, BackgroundComponent, ChoiceCursorComponent, ChoiceItemComponent};

const DEFAULT_FONT_PATH: &str = "fonts/Silver.ttf";
const DEFAULT_FONT_SIZE: f32 = 45.;
const DEFAULT_TEXT_COLOR: Color = Color::rgba(1., 1., 1., 1.);
const DEFAULT_CURSOR_PATH: &str = "images/cursor.png";
const DEFAULT_BOX_SIZE: Vec2 = Vec2::new(350., 600.);
const DEFAULT_BOX_POSITION: Vec3 = Vec3::new(-600., 100., 1.);
const DEFAULT_TEXT_SPEED: f32 = 100.;
const DEFAULT_VOICE_FREQUENCY: f32 = 0.1; 
const DEFAULT_CHOICE_SPACING: f32 = 40.;
const DEFAULT_CHOICE_INDENT: f32 = 25.;
const DEFAULT_CURSOR_OFFSET: f32 = 16.;
const DEFAULT_PORTRAIT_POSITION: Vec3 = Vec3::new(-425., 225., 1.);

pub struct BirdsongPlugin;

static INIT: &str = "init";

impl Plugin for BirdsongPlugin {
    fn build(&self, app:&mut App) {
        app
            .add_startup_stage_before(StartupStage::Startup, INIT,  SystemStage::single_threaded())
            .add_startup_system_to_stage(INIT, birdsong_setup_default_settings_system)
            .add_startup_system(birdsong_setup_system)
            .add_system(birdsong_parse_script_system)
            .add_system(birdsong_handle_input_system)
            .add_system(birdsong_process_entry_system)
            .add_system(birdsong_update_dialoguebox_system)
            .add_system(birdsong_update_choices_system)
            .add_system(birdsong_update_background_system)
            .add_system(birdsong_update_actor_system)
            .add_system(birdsong_update_exposed_line_system);
        }
}

pub struct Birdsong {
    script_data: ScriptData,
    curr_line: usize
}

impl Birdsong {
    pub fn start(&mut self, script: String) {
        self.script_data.script = script;
        self.script_data.updated = true;
    }

    pub fn get_curr_line(&mut self) -> usize {
        return self.curr_line;
    }
}

struct SettingsData {
    text_style: TextStyle,
    cursor_sprite: Handle<Image>,
    box_size: Vec2,
    box_position: Vec3,
    box_text_speed: f32,
    voice_frequency: f32,
    choice_spacing: f32,
    choice_indent: f32,
    cursor_offset: f32,
    portrait_position:Vec3,
}

struct ScriptData {
    script: String,
    updated: bool,
}

struct EntriesData {
    updated: bool,
    list: Vec<[String; 2]>,
}

struct FontsData {
    font_map: HashMap<String, Handle<Font>>,
}

struct ChoicesData {
    enabled: bool,
    created: bool,
    updated: bool,
    curr_choice: i32,
    next: i32,
    size: i32,
    cursor_sprite_map: HashMap<String, Handle<Image>>,
}

struct DialogueBoxData {
    enabled: bool,
    created: bool,
    updated: bool,
    cursor: f32,
    entry: String,
    entry_num: usize,
    is_printing: bool,
}

struct ActorsData {
    enabled: bool,
    created: bool,
    updated: bool,
    portraits_map: HashMap<String, Handle<Image>>,
    voices_map: HashMap<String, Handle<AudioSource>>,
    curr_name: String,
    voice_timer: Timer,
}

struct BackgroundsData {
    enabled: bool,
    created: bool,
    updated: bool,
    map: HashMap<String, BackgroundImageData>,
    curr_name: String
}

struct BackgroundImageData {
    pos: Vec2,
    handle: Handle<Image>,
}

fn birdsong_setup_default_settings_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(DEFAULT_FONT_PATH);
    let text_style = TextStyle {
        font,
        font_size: DEFAULT_FONT_SIZE,
        color: DEFAULT_TEXT_COLOR,
    };    
    let settings = SettingsData{text_style: text_style, cursor_sprite: asset_server.load(DEFAULT_CURSOR_PATH), box_size: DEFAULT_BOX_SIZE, box_position: DEFAULT_BOX_POSITION, box_text_speed: DEFAULT_TEXT_SPEED, voice_frequency: DEFAULT_VOICE_FREQUENCY, choice_spacing: DEFAULT_CHOICE_SPACING, choice_indent: DEFAULT_CHOICE_INDENT, cursor_offset: DEFAULT_CURSOR_OFFSET, portrait_position:DEFAULT_PORTRAIT_POSITION};
    commands.insert_resource(settings);
}

fn birdsong_setup_system(mut commands: Commands, settings: ResMut<SettingsData>) {
    let birdsong = Birdsong{script_data: ScriptData{script:"".to_string(), updated:true}, curr_line: 0};
    commands.insert_resource(birdsong);

    let fonts = FontsData{font_map: HashMap::new()};
    commands.insert_resource(fonts);

    let entries = EntriesData{updated: false, list: Vec::new()};
    commands.insert_resource(entries);

    let dialoguebox = DialogueBoxData{enabled: false, created: false, updated: true,  cursor: 0.0, entry: "".to_string(), entry_num: 0, is_printing: false};
    commands.insert_resource(dialoguebox);

    let actors = ActorsData{enabled: false, created: false, updated: true, portraits_map: HashMap::new(), voices_map: HashMap::new(), curr_name:"none".to_string(), voice_timer: Timer::new(Duration::from_secs_f32(settings.voice_frequency), true)};
    commands.insert_resource(actors);

    let choices = ChoicesData{enabled: false, created: false, updated: true, curr_choice: 0, next: 0, size: 0, cursor_sprite_map: HashMap::new()};
    commands.insert_resource(choices);

    let backgrounds = BackgroundsData{enabled: false, created: false, updated: true, map: HashMap::new(), curr_name:"none".to_string()};
    commands.insert_resource(backgrounds);    
}

fn birdsong_parse_script_system(asset_server: Res<AssetServer>, mut birdsong: ResMut<Birdsong>, mut fonts: ResMut<FontsData>, mut actors: ResMut<ActorsData>, mut backgrounds: ResMut<BackgroundsData>, mut entries: ResMut<EntriesData>, mut choices: ResMut<ChoicesData>) {
    if birdsong.script_data.updated {
        birdsong.script_data.updated = false;
        let mut state = 0; // 0: Sprites, 1: Voices, 2: Lines

        let script_str = birdsong.script_data.script.as_str();
        for line in script_str.lines() {
            match line {
                "## FONTS" => {
                    state = 0;
                    continue;
                },
                "## CURSOR SPRITES" => {
                    state = 1;
                    continue;
                },
                "## BACKGROUNDS" => {
                    state = 2;
                    continue;
                },
                "## ACTORS" => {
                    state = 3;
                    continue;
                },
                "## ENTRIES" => {
                    state = 4;
                    continue;
                },
                "" => continue,
                _ => (),
            }
            let line_vec: Vec<&str> = line.split("#").collect();
            match state {
                0 => {
                    fonts.font_map.insert(line_vec[0].to_string(), asset_server.load(line_vec[1]));
                },
                1 => {
                    choices.cursor_sprite_map.insert(line_vec[0].to_string(), asset_server.load(line_vec[1]));
                },
                2 => {
                    let bg_vec: Vec<&str> = line_vec[1].split("@").collect();
                    let bg_size_vec: Vec<&str> = bg_vec[1].split("x").collect();
                    let bg_pos = Vec2::new(bg_size_vec[0].parse().unwrap(), bg_size_vec[1].parse().unwrap()); 
                    let bg = asset_server.load(bg_vec[0]);
                    backgrounds.map.insert(line_vec[0].to_string(), BackgroundImageData{pos:bg_pos, handle:bg});
                },
                3 => {
                    let actor_vec: Vec<&str> = line_vec[1].split("|").collect();
                    let sprite = asset_server.load(actor_vec[0]); 
                    actors.portraits_map.insert(line_vec[0].to_string(), sprite);
                    let voice = asset_server.load(actor_vec[1]); 
                    actors.voices_map.insert(line_vec[0].to_string(), voice);
                },
                4 => {
                    entries.list.push([line_vec[0].to_string(), line_vec[1].to_string()]);
                },
                _ => (),
            }
                
        }
        entries.updated = true;
    }
}

fn birdsong_process_entry_system(mut settings: ResMut<SettingsData>, mut dbox: ResMut<DialogueBoxData>, fonts: Res<FontsData>, mut choices: ResMut<ChoicesData>, entries: Res<EntriesData>, mut actors: ResMut<ActorsData>,  mut backgrounds: ResMut<BackgroundsData>) {  
    let entry_num = entries.list.len();
    if entry_num > 0 && entries.updated {
        let entry_type = entries.list[dbox.entry_num][0].clone();
        if dbox.entry_num < entry_num {
            match entry_type.as_str() {
                "s" => {
                    let settings_vec: Vec<&str> = entries.list[dbox.entry_num][1].split("|").collect();
                    for setting in settings_vec {
                        let setting_vec: Vec<&str> = setting.split(":").collect();
                        match setting_vec[0] {
                            "font" => {
                                settings.text_style.font = fonts.font_map.get(setting_vec[1]).unwrap().clone();
                            },
                            "font_size" => {
                                settings.text_style.font_size = setting_vec[1].parse::<f32>().unwrap();
                            },
                            "font_color" => {
                                let text_color_vec: Vec<&str> = setting_vec[1].split("x").collect();
                                settings.text_style.color = Color::Rgba{red: text_color_vec[0].parse::<f32>().unwrap(), green: text_color_vec[1].parse::<f32>().unwrap(), blue: text_color_vec[2].parse::<f32>().unwrap(), alpha: text_color_vec[3].parse::<f32>().unwrap()}
                            },
                            "cursor" => {
                                settings.cursor_sprite = choices.cursor_sprite_map.get(setting_vec[1]).unwrap().clone();
                            },
                            "box_size" => {
                                let box_size_vec: Vec<&str> = setting_vec[1].split("x").collect();
                                settings.box_size = Vec2::new(box_size_vec[0].parse::<f32>().unwrap(), box_size_vec[1].parse::<f32>().unwrap());
                            },
                            "box_position" => {
                                let box_position_vec: Vec<&str> = setting_vec[1].split("x").collect();
                                settings.box_position = Vec3::new(box_position_vec[0].parse::<f32>().unwrap(), box_position_vec[1].parse::<f32>().unwrap(), box_position_vec[2].parse::<f32>().unwrap());
                            }
                            "box_text_speed" => {
                                settings.box_text_speed = setting_vec[1].parse::<f32>().unwrap();
                            },
                            "voice_frequency" => {
                                settings.voice_frequency = setting_vec[1].parse::<f32>().unwrap();
                            },
                            "choice_spacing" => {
                                settings.choice_spacing = setting_vec[1].parse::<f32>().unwrap();
                            },
                            "choice_indent" => {
                                settings.choice_indent = setting_vec[1].parse::<f32>().unwrap();
                            },
                            "cursor_offset" => {
                                settings.cursor_offset = setting_vec[1].parse::<f32>().unwrap();
                            },
                            "portrait_position" => {
                                let portrait_position_vec: Vec<&str> = setting_vec[1].split("x").collect();
                                settings.portrait_position = Vec3::new(portrait_position_vec[0].parse::<f32>().unwrap(), portrait_position_vec[1].parse::<f32>().unwrap(), portrait_position_vec[2].parse::<f32>().unwrap());
                            },
                            _ => {},
                        }
                    }
                    dbox.entry_num += 1;
                }
                "c" => {
                    if dbox.enabled {
                        dbox.enabled = false;
                    }
                    if actors.enabled {
                        actors.enabled = false;
                    }
                    if !choices.enabled {
                        choices.enabled = true;
                    }
                    choices.updated = false;
                },
                "t" => {
                    if !dbox.enabled {
                        dbox.enabled = true;
                    } 
                    dbox.updated = false;
                    
                    let entry_vec: Vec<&str> = entries.list[dbox.entry_num][1].split("@").collect();
                    match entry_vec.len() {
                        1 => {
                            dbox.entry = entry_vec[0].to_string();
                        },
                        2 => {
                            actors.curr_name = entry_vec[0].to_string();
                            actors.updated = false;
                            if !actors.enabled {
                                actors.enabled = true;
                            }
                            dbox.entry = entry_vec[1].to_string();
                        },
                        _ => {}
                    }
                },
                "i" => {
                    if !backgrounds.enabled {
                        backgrounds.enabled = true;
                    }
                    backgrounds.updated = false;
                    backgrounds.curr_name = entries.list[dbox.entry_num][1].clone();
                    dbox.entry_num += 1;
                },
                _ => {
                    //actors.curr_name = "none".to_string();
                }
            }
        }
    }
}

fn birdsong_handle_input_system(kb: Res<Input<KeyCode>>, mut dbox: ResMut<DialogueBoxData>, mut entries: ResMut<EntriesData>, mut choices: ResMut<ChoicesData>) {
    if kb.just_pressed(KeyCode::Space) || kb.just_pressed(KeyCode::Return) {
        if choices.enabled {
            dbox.entry_num = choices.next as usize;
            dbox.cursor = 0.;
            choices.enabled = false;
        }
        else {
            if dbox.entry_num < entries.list.len() {
                if dbox.is_printing {
                    dbox.cursor = entries.list[dbox.entry_num][1].len() as f32;
                }
                else if dbox.entry_num < entries.list.len()-1 {
                    dbox.entry_num += 1;
                    dbox.cursor = 0.;
                }
            }
            else {
                entries.updated = false;
            }
        }
    }
    if kb.just_pressed(KeyCode::Up) || kb.just_pressed(KeyCode::Z) || kb.just_pressed(KeyCode::W) {
        if choices.curr_choice > 0 {
            choices.curr_choice -= 1;
            choices.updated = false;
        }
    }
    if kb.just_pressed(KeyCode::Down) || kb.just_pressed(KeyCode::S) {
        if choices.curr_choice < choices.size - 1 {
            choices.curr_choice += 1;
            choices.updated = false;
        }
    }

}

fn birdsong_update_dialoguebox_system(mut commands: Commands, time: Res<Time>, settings: Res<SettingsData>, mut dbox: ResMut<DialogueBoxData>, mut query: Query<(Entity, &mut DialogueBoxComponent, &mut Text)>) {
    if dbox.enabled && !dbox.created {
        dbox.created = true;
        commands.spawn_bundle(Text2dBundle {
            text: Text::from_section("", settings.text_style.clone()),
            text_2d_bounds: Text2dBounds {
                size: settings.box_size,
            },
            transform: Transform::from_translation(settings.box_position),
            ..default()
        })
        .insert(DialogueBoxComponent);
    }
    for(ent, _, mut text) in query.iter_mut() {
        if !dbox.enabled && dbox.created {
            dbox.created = false;
            dbox.updated = true;
            commands.entity(ent).despawn();
        }
        else if !dbox.updated {
            let curr_cursor = dbox.cursor.floor() as usize;
            if curr_cursor < dbox.entry.len() {
                dbox.is_printing = true;
                let curr_slice = &dbox.entry[..curr_cursor];
                text.sections[0].value = curr_slice.to_string();
                dbox.cursor+=settings.box_text_speed*time.delta_seconds();
            } else {
                dbox.is_printing = false;
                text.sections[0].value = dbox.entry.clone();
            }
        }
    }
    dbox.updated = true;
}

fn birdsong_update_choices_system(mut commands: Commands, entries: Res<EntriesData>, dbox: Res<DialogueBoxData>, settings: Res<SettingsData>, mut choices: ResMut<ChoicesData>, mut cursor_query: Query<(&ChoiceCursorComponent, &mut Visibility)>, mut entity_query: Query<(Entity, &ChoiceItemComponent)>) {
    if choices.enabled && !choices.created {
        choices.created = true;
        let choices_list = entries.list[dbox.entry_num][1].split("|");
        let mut curr_delta = 0.;
        let mut count = 0;
        choices.curr_choice = 0;

        for choice in choices_list {
            let choice_vec: Vec<&str> = choice.split("@").collect();
            let choice_pos = settings.box_position + Vec3::new(0., -curr_delta, 0.);

            commands.spawn_bundle(Text2dBundle {
                text: Text::from_section(choice_vec[0], settings.text_style.clone()),
                text_2d_bounds: Text2dBounds {
                    size: settings.box_size,
                },
                transform: Transform::from_translation(settings.box_position + Vec3::new(settings.choice_indent, -curr_delta, 0.)),
                ..default()
            }
            )
            .insert(ChoiceItemComponent);
            
            commands.spawn_bundle(SpriteBundle {
                texture: settings.cursor_sprite.clone(),
                transform: Transform {
                    translation: choice_pos + Vec3::new(0., -settings.cursor_offset, 0.),
                    ..default()
                },
                visibility: Visibility { is_visible: false },
                ..default()
            })
            .insert(ChoiceCursorComponent{num: count, next: choice_vec[1].to_string().parse::<i32>().unwrap(), anchor: choice_pos})
            .insert(ChoiceItemComponent);
            curr_delta += settings.choice_spacing;
            choices.size += 1;
            count += 1;
        }
    }

    if !choices.enabled && choices.created {
        for(ent, _) in entity_query.iter_mut() {
            choices.created = false;
            choices.updated = true;
            commands.entity(ent).despawn();
        }
    }
    else if !choices.updated {
        for(cursor, mut vis) in cursor_query.iter_mut() {
            if cursor.num == choices.curr_choice {
                choices.next = cursor.next;
                vis.is_visible = true;
            }
            else {
                vis.is_visible = false;
            }
        }
    }
    choices.updated = true;
}

fn birdsong_update_actor_system(time: Res<Time>, mut commands: Commands, dbox: Res<DialogueBoxData>, audio: Res<Audio>, mut actors: ResMut<ActorsData>, settings: Res<SettingsData>, mut query: Query<(Entity, &mut Handle<Image>, &mut Transform, &PortraitComponent)>) {
    if actors.enabled && !actors.created {
        actors.created = true;
        commands.spawn_bundle(SpriteBundle {
            texture: actors.portraits_map.get(&actors.curr_name).unwrap().clone(),
            transform: Transform {
                translation: settings.portrait_position,
                ..default()
            },
            ..default()
        })
        .insert(PortraitComponent)
        .insert(VoiceComponent);
    }
    for(ent, mut sprite, mut transform, _) in query.iter_mut() {
        if !actors.enabled && actors.created {
            actors.created = false;
            commands.entity(ent).despawn();
        }
        else if !actors.updated {
            *sprite = actors.portraits_map.get(&actors.curr_name).unwrap().clone();
            transform.translation = settings.portrait_position;
        }
    }
    if actors.enabled {
        if actors.voice_timer.duration().as_secs_f32() != settings.voice_frequency {
            actors.voice_timer = Timer::new(Duration::from_secs_f32(settings.voice_frequency), true);
        }
        actors.voice_timer.tick(time.delta());
        if actors.voice_timer.just_finished() && dbox.is_printing {
            audio.play(actors.voices_map.get(&actors.curr_name).unwrap().clone());
        }
    }
    actors.updated = true;
}

fn birdsong_update_background_system(mut commands: Commands, mut backgrounds: ResMut<BackgroundsData>, mut query: Query<(Entity, &mut Handle<Image>, &mut Transform, &BackgroundComponent)>) {
    if backgrounds.enabled && !backgrounds.created {
        backgrounds.created = true;
        let bg = backgrounds.map.get(&backgrounds.curr_name).unwrap();
        println!("{}", bg.pos.x);
        commands.spawn_bundle(SpriteBundle {
            texture: bg.handle.clone(),
            transform: Transform {
                translation: Vec3::new(bg.pos.x, bg.pos.y, 0.),
                ..default()
            },
            ..default()
        })
        .insert(BackgroundComponent);
    }
    for(ent, mut sprite, mut transform, _) in query.iter_mut() {
        if !backgrounds.enabled && backgrounds.created {
            backgrounds.created = false;
            commands.entity(ent).despawn();
        }
        else if !backgrounds.updated {
            let bg = backgrounds.map.get(&backgrounds.curr_name).unwrap();
            *sprite = bg.handle.clone();
            transform.translation = Vec3::new(bg.pos.x, bg.pos.y, 0.);
        }
    }
    backgrounds.updated = true;
}

fn birdsong_update_exposed_line_system(mut birdsong: ResMut<Birdsong>, dbox: Res<DialogueBoxData>) {
    birdsong.curr_line = dbox.entry_num;
}