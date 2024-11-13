use crate::IntoDocItemAttribute;

pub struct UserAttribute;

impl IntoDocItemAttribute for UserAttribute {
    fn prefix(&self) -> String {
        "have:".to_string()
    }
}
