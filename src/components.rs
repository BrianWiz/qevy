use bevy::prelude::*;
use std::{collections::BTreeMap, time::Duration};

#[derive(Default, Component)]
pub struct Map {
    pub asset: Handle<crate::MapAsset>,
}

#[derive(Default, Bundle)]
pub struct MapBundle {
    pub map: Map,
    pub transform: TransformBundle,
    pub visibility: VisibilityBundle,
}

#[derive(Default, Component)]
pub struct MapEntityProperties {
    pub classname: String,
    pub transform: Transform,
    pub properties: BTreeMap<String, String>,
}

impl MapEntityProperties {
    // FGD format: https://developer.valvesoftware.com/wiki/FGD

    pub fn get_property_as_string(&self, key: &str, default: Option<&String>) -> Option<String> {
        if let Some(value) = self.properties.get(key) {
            return Some(value.clone());
        }
        default.cloned()
    }

    pub fn get_property_as_f32(&self, key: &str, default: f32) -> f32 {
        if let Some(value) = self.properties.get(key) {
            if let Ok(value) = value.parse::<f32>() {
                return value;
            }
        }
        default
    }

    pub fn get_property_as_i32(&self, key: &str, default: i32) -> i32 {
        if let Some(value) = self.properties.get(key) {
            if let Ok(value) = value.parse::<i32>() {
                return value;
            }
        }
        default
    }

    pub fn get_property_as_bool(&self, key: &str, default: bool) -> bool {
        if let Some(value) = self.properties.get(key) {
            if let Ok(value) = value.parse::<i32>() {
                if value == 1 {
                    return true;
                } else {
                    return false;
                }
            }
        }
        default
    }

    pub fn get_property_as_color(&self, key: &str, default: Color) -> Color {
        if let Some(value) = self.properties.get(key) {
            let value = value.trim().split(" ").collect::<Vec<&str>>();
            if value.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    value[0].parse::<f32>(),
                    value[1].parse::<f32>(),
                    value[2].parse::<f32>(),
                ) {
                    return Color::rgb(r, g, b);
                }
            }
        }
        default
    }

    pub fn get_property_as_vec3(&self, key: &str, default: Vec3) -> Vec3 {
        if let Some(value) = self.properties.get(key) {
            let value = value.trim().split(" ").collect::<Vec<&str>>();
            if value.len() == 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    value[0].parse::<f32>(),
                    value[1].parse::<f32>(),
                    value[2].parse::<f32>(),
                ) {
                    return Vec3::new(x, y, z);
                }
            }
        }
        default
    }
}

#[derive(Default, Component)]
pub struct BrushEntity;

#[derive(Default, Component)]
pub struct Brush;

#[derive(Component)]
pub struct TriggeredOnce;

#[derive(Event)]
pub struct TriggeredEvent {
    pub target: String,
    pub triggered_by: Entity,
}

#[derive(Default, Component)]
pub struct TriggerOnce {
    pub target: String,
}

#[derive(Default, Component)]
pub struct TriggerMultiple {
    pub target: String,
}

#[derive(Default, Component)]
pub struct TriggerTarget {
    pub target_name: String,
}

#[derive(Default, Component)]
pub struct TriggerInstigator;

#[derive(Default, Component)]
pub struct Mover {
    pub state: MoverState,
    /// time it takes to move from start to destination and vice versa
    pub moving_time: Duration,
    /// time it takes to stay at the destination
    pub destination_time: Duration,
    /// the offset from the start position
    pub destination_offset: Vec3,
}

#[derive(Default)]
pub enum MoverState {
    #[default]
    AtStart,
    MovingToDestination(Timer),
    AtDestination(Timer),
    MovingToStart(Timer),
}

impl MoverState {
    pub fn get_fraction(&self) -> f32 {
        match self {
            Self::AtStart => 0.0,
            Self::MovingToDestination(timer) => timer.fraction().min(1.0).max(0.0),
            Self::MovingToStart(timer) => timer.fraction().min(1.0).max(0.0),
            Self::AtDestination(timer) => timer.fraction().min(1.0).max(0.0),
        }
    }
}

#[derive(Component, Default)]
pub struct Door {
    /// the key required to open the door
    pub key: Option<String>,
    /// whether the door should open only once and stay open
    pub open_once: bool,
}

/// The units used in the map
/// Bevy units are the default units used in Bevy, which are 1 unit = 1 meter
/// Trenchbroom units are the units used in Trenchbroom, which are 16 units = 1 foot
#[derive(Resource, Clone)]
pub enum MapUnits {
    Bevy,
    Trenchbroom,
}

impl Default for MapUnits {
    fn default() -> Self {
        Self::Bevy
    }
}
