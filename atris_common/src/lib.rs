use std::{borrow::Borrow, fmt::Display, marker::PhantomData};

pub use chacha20poly1305::{self as cipher};
use chacha20poly1305::{
    aead::{self, Aead, OsRng},
    AeadCore, ChaCha20Poly1305, Nonce,
};
use cipher::KeyInit;
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

pub mod authenticate_user;
pub mod create_room;
pub mod create_user;
pub mod join_room;
pub mod set_room_responder;

pub type Cipher = ChaCha20Poly1305;
#[derive(Debug, Clone)]
pub struct CipherKey(cipher::Key);

impl CipherKey {
    pub fn generate() -> Self {
        Cipher::generate_key(&mut OsRng).into()
    }
    pub fn as_cipher(&self)->Cipher {
        Cipher::new(&self.0)
    }
}
impl From<cipher::Key> for CipherKey {
    fn from(key: cipher::Key) -> Self {
        CipherKey(key)
    }
}
impl From<&[u8]> for CipherKey {
    fn from(bytes: &[u8]) -> Self {
        CipherKey(cipher::Key::clone_from_slice(&bytes))
    }
}
impl AsRef<[u8]> for CipherKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Borrow<cipher::Key> for CipherKey {
    fn borrow(&self) -> &cipher::Key {
        &self.0
    }
}
impl Serialize for CipherKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // let bytes = &*self.0;
        // let mut a = serializer.serialize_seq(Some(bytes.len()))?;
        // for b in bytes {
        //     a.serialize_element(b)?;
        // }
        // a.end()
        serializer.serialize_bytes(self.as_ref())
    }
}

pub struct CipherKeyVisitor;
impl<'de> Visitor<'de> for CipherKeyVisitor {
    type Value = CipherKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte buffer")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut bytes = Vec::new();
        while let Ok(Some(b)) = seq.next_element::<u8>() {
            bytes.push(b)
        }
        Ok((bytes.as_slice()).into())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CipherKey(cipher::Key::clone_from_slice(&v)))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CipherKey(cipher::Key::clone_from_slice(&v)))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CipherKey(cipher::Key::clone_from_slice(&v)))
    }
}

impl<'de> Deserialize<'de> for CipherKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(CipherKeyVisitor)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoomData {
    /// The responder to attatch to to
    pub responder_string: String,
    /// The symmetric key the conversation will be encrypted with
    pub symmetric_key: CipherKey,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Encrypted<T> {
    #[serde(with = "serde_bytes")]
    nonce: Vec<u8>,

    #[serde(with = "serde_bytes")]
    cipher_bytes: Vec<u8>,

    #[serde(skip)]
    phantom_data: PhantomData<T>,
}
#[derive(Debug)]
pub enum EncryptionError {
    BincodeError(bincode::Error),
    AEADError(aead::Error),
}
impl From<bincode::Error> for EncryptionError {
    fn from(err: bincode::Error) -> Self {
        Self::BincodeError(err)
    }
}

impl From<aead::Error> for EncryptionError {
    fn from(err: aead::Error) -> Self {
        Self::AEADError(err)
    }
}
pub type Result<T> = std::result::Result<T, EncryptionError>;

impl<T: Serialize + for<'de> Deserialize<'de>> Encrypted<T> {
    pub fn encrypt(value: &T, cipher: &mut ChaCha20Poly1305) -> self::Result<Self> {
        let value_bytes = bincode::serialize(value)?;
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let cipher_bytes = cipher.encrypt(&nonce, value_bytes.as_ref())?;
        Ok(Self {
            nonce: nonce.to_vec(),
            cipher_bytes,
            phantom_data: PhantomData,
        })
    }

    pub fn decrypt(self, cipher: &mut ChaCha20Poly1305) -> self::Result<T> {
        // let nonce_new: Nonce = (&c.nonce).into_iter().cloned().collect();
        let nonce: Nonce = self.nonce.into_iter().collect();
        let value_bytes: Vec<u8> = cipher.decrypt(&nonce, self.cipher_bytes.as_ref())?;
        let value = bincode::deserialize(&value_bytes)?;
        Ok(value)
    }
}

// Error enum
#[derive(Debug, PartialEq, Eq)]
pub enum AtrisError {
    SerdeError(String),
}
impl Display for AtrisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerdeError(err) => {
                write!(f, "Serde Error: {}", err)
            }
        }
    }
}
/// The region for Atris on Lambda
pub const REGION: &'static str = "us-west-2";
