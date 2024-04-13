/// This crate contains the types used by the qevy crate. Needed because i need it in the derive crate as well as the normal one.

#[derive(Debug, PartialEq)]
pub enum QevyEntityType {
    Base,
    Solid,
    Point,
}

impl QevyEntityType {
    pub fn to_fgd_string(&self) -> &str {
        match self {
            QevyEntityType::Base => "@BaseClass",
            QevyEntityType::Solid => "@SolidClass",
            QevyEntityType::Point => "@PointClass",
        }
    }

    pub fn from_short_string(string: &str) -> Option<Self> {
        match string {
            "Base" => Some(QevyEntityType::Base),
            "Solid" => Some(QevyEntityType::Solid),
            "Point" => Some(QevyEntityType::Point),
            _ => None,
        }
    }
}
