use bevy::prelude::Component;
use bevy::math::Vec3;

#[derive(Component)]
pub struct DialogueBoxComponent;

#[derive(Component)]
pub struct PortraitComponent;

#[derive(Component)]
pub struct VoiceComponent;

#[derive(Component)]
pub struct BackgroundComponent;

#[derive(Component)]
pub struct ChoiceCursorComponent{
    pub num: i32,
    pub next: i32,
    pub anchor: Vec3,
}

#[derive(Component)]
pub struct ChoiceItemComponent;