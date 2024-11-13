use crate::IntoDocItemAttribute;

pub struct UserAttribute;

impl IntoDocItemAttribute for UserAttribute {
    fn prefix(&self) -> String {
        "ensure:".to_string()
    }
}
