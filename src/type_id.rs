#[repr(u8)]
pub enum TypeIdentifier {
    NIL = 0x2D,    // '-' (45)
    FALSE = 0x30,  // '0' (48)
    TRUE = 0x31,   // '1' (49)
    NUMBER = 0x4E, // 'N' (78)
    STRING = 0x53, // 'S' (83)
    TABLE = 0x54,  // 'T' (84)
}

impl std::convert::TryFrom<u8> for TypeIdentifier {
    type Error = nom::error::ErrorKind;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == TypeIdentifier::NIL as u8 {
            Ok(TypeIdentifier::NIL)
        } else if value == TypeIdentifier::FALSE as u8 {
            Ok(TypeIdentifier::FALSE)
        } else if value == TypeIdentifier::TRUE as u8 {
            Ok(TypeIdentifier::TRUE)
        } else if value == TypeIdentifier::NUMBER as u8 {
            Ok(TypeIdentifier::NUMBER)
        } else if value == TypeIdentifier::STRING as u8 {
            Ok(TypeIdentifier::STRING)
        } else if value == TypeIdentifier::TABLE as u8 {
            Ok(TypeIdentifier::TABLE)
        } else {
            Err(nom::error::ErrorKind::Digit)
        }
    }
}
