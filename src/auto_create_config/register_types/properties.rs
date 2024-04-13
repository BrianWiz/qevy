use bevy::prelude::*;

pub use qevy_derive::QevyProperty;

pub(crate) struct QevyPropertyPlugin;

// Macro for registering type data for each type implemented QevyProperty
macro_rules! register_qevy_property_types {
    ($app:expr, $($t:ty),*) => {
        $(
            $app.register_type_data::<$t, ReflectQevyProperty>();
        )*
    };
}

impl Plugin for QevyPropertyPlugin {
    fn build(&self, app: &mut App) {
        // Register all types that implement QevyProperty here
        register_qevy_property_types!(
            app, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f64, f32, String,
            bool, Color
        );
        app.register_type::<QevyAngles>();
    }
}

#[reflect_trait]
pub trait QevyProperty: Reflect {
    fn get_fgd_string(&self, field_name: &str, field_description: &str) -> &'static str;
}

#[derive(Reflect, Default)]
#[reflect(QevyProperty)]
pub struct QevyAngles {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl QevyProperty for QevyAngles {
    fn get_fgd_string(&self, _field_name: &str, field_description: &str) -> &'static str {
        Box::leak(
            format!(
                "\tangles(string) : \"Angles (x y z)\" : \"{} {} {}\" : \"{}\"",
                self.pitch, self.yaw, self.roll, field_description
            )
            .into_boxed_str(),
        )
    }
}

// Implementation for bool
impl QevyProperty for bool {
    fn get_fgd_string(&self, field_name: &str, field_description: &str) -> &'static str {
        let value = if *self { 1 } else { 0 };

        Box::leak(
            format!(
            "\t{}(choices) : \"{}\" : {} : \"{}\" =\n\t[\n\t\t0 : \"False\"\n\t\t1 : \"True\"\n\t]",
            field_name, field_name, value, field_description
        )
            .into_boxed_str(),
        )
    }
}

// Implementation for color
impl QevyProperty for Color {
    fn get_fgd_string(&self, field_name: &str, field_description: &str) -> &'static str {
        let rgb_string = format!(
            "{} {} {}",
            (self.r() * 255.0) as u8,
            (self.g() * 255.0) as u8,
            (self.b() * 255.0) as u8
        );

        Box::leak(
            format!(
                "\t{}(color) : \"{}\" : \"{}\" : \"{}\"",
                field_name, field_name, rgb_string, field_description
            )
            .into_boxed_str(),
        )
    }
}

// Macro to implement QevyProperty for given types
macro_rules! impl_qevy_property {
    ($label:expr, $quote:expr, $($t:ty),*) => {
        $(
            impl QevyProperty for $t {
                fn get_fgd_string(&self, field_name: &str, field_description: &str) -> &'static str {
                    let formatted_value = if $quote {
                        format!("\"{}\"", self)
                    } else {
                        format!("{}", self)
                    };

                    Box::leak(
                        format!("\t{}({}) : \"{}\" : {} : \"{}\"", field_name, $label, field_name, formatted_value, field_description)
                            .into_boxed_str()
                    )
                }
            }
        )*
    };
}

// Implement the trait for primitive types
impl_qevy_property!(
    "integer", false, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
impl_qevy_property!("float", true, f64, f32);
impl_qevy_property!("string", true, String);
