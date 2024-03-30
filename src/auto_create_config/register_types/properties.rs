use bevy::prelude::*;

pub(crate) struct QevyPropertyPlugin;

impl Plugin for QevyPropertyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type_data::<u8, ReflectQevyProperty>();
    }
}

#[reflect_trait]
pub trait QevyProperty: Reflect {
    fn get_fgd_string(&self, field_name: &str) -> String;
}

impl QevyProperty for bool {
    fn get_fgd_string(&self, field_name: &str) -> String {
        let value = if *self { 1 } else { 0 };
        format!(
            "{}(choices) : \"{}\" : {} = [0 : \"False\" 1 : \"True\"]",
            field_name, field_name, value
        )
    }
}

macro_rules! impl_qevy_property {
    ($label:expr, $quote:expr, $($t:ty),*) => {
        $(
            impl QevyProperty for $t {
                fn get_fgd_string(&self, field_name: &str) -> String {
                    if $quote {
                        format!("{}({}) : \"{}\" : \"{}\" : \"Placeholder Description\"", field_name, field_name, $label, self)
                    } else {
                        format!("{}({}) : \"{}\" : {} : \"Placeholder Description\"", field_name, field_name, $label, self)
                    }
                }
            }
        )*
    };
}

// Impl the trait for primitive types
impl_qevy_property!(
    "integer", false, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
impl_qevy_property!("float", true, f64, f32);
impl_qevy_property!("string", true, String);
