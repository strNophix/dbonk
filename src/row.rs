use crate::layout::*;
use core::fmt::Debug;

pub type RowBytes = [u8; ROW_SIZE];

pub struct Row {
    id: u16,
    username: [u8; 32],
    email: [u8; 255],
}

impl Row {
    pub fn new(id: u16, username: &str, email: &str) -> Self {
        let mut row = Self {
            id,
            ..Default::default()
        };

        row.username[..username.len()].copy_from_slice(username.as_bytes());
        row.email[..email.len()].copy_from_slice(email.as_bytes());

        row
    }
}

impl Default for Row {
    fn default() -> Self {
        Self {
            id: 0,
            username: [0; 32],
            email: [0; 255],
        }
    }
}

impl From<RowBytes> for Row {
    fn from(value: RowBytes) -> Self {
        let id = u16::from_be_bytes([value[0], value[1]]);
        let username: [u8; USERNAME_SIZE] =
            value[USERNAME_OFFSET..EMAIL_OFFSET].try_into().unwrap();
        let email: [u8; EMAIL_SIZE] = value[EMAIL_OFFSET..ROW_SIZE].try_into().unwrap();
        Self {
            id,
            username,
            email,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<RowBytes> for Row {
    fn into(self) -> RowBytes {
        let mut bytes = [0u8; ROW_SIZE];
        bytes[..USERNAME_OFFSET].copy_from_slice(&self.id.to_be_bytes());
        bytes[USERNAME_OFFSET..EMAIL_OFFSET].copy_from_slice(&self.username);
        bytes[EMAIL_OFFSET..ROW_SIZE].copy_from_slice(&self.email);
        bytes
    }
}

impl Debug for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let username = std::str::from_utf8(&self.username).expect("invalid utf8");
        let email = std::str::from_utf8(&self.email).expect("invalid utf8");
        f.debug_struct("Row")
            .field("id", &self.id)
            .field("username", &username.trim_end_matches('\0'))
            .field("email", &email.trim_end_matches('\0'))
            .finish()
    }
}
