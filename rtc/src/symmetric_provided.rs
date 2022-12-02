use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};

pub fn main() {
    // User 1
    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    let mut cipher = ChaCha20Poly1305::new(&key);
    let message = "I'd just like to interject for a moment. What you're refering to as Linux, is in fact, GNU/Linux, or as I've recently taken to calling it, GNU plus Linux. Linux is not an operating system unto itself, but rather another free component of a fully functioning GNU system made useful by the GNU corelibs, shell utilities and vital system components comprising a full OS as defined by POSIX.
Many computer users run a modified version of the GNU system every day, without realizing it. Through a peculiar turn of events, the version of GNU which is widely used today is often called Linux, and many of its users are not aware that it is basically the GNU system, developed by the GNU Project.
There really is a Linux, and these people are using it, but it is just a part of the system they use. Linux is the kernel: the program in the system that allocates the machine's resources to the other programs that you run. The kernel is an essential part of an operating system, but useless by itself; it can only function in the context of a complete operating system. Linux is normally used in combination with the GNU operating system: the whole system is basically GNU with Linux added, or GNU/Linux. All the so-called Linux distributions are really distributions of GNU/Linux!";
    println!("Sent message:\n{:?}", message);
    let message_encrypted = encrypt(message, &mut cipher).expect("Cipher error");
    let serialized_message = bincode::serialize(&message_encrypted).expect("Serialization error");
    println!("====================================");
    // User 2
    let deserialized_encrypted_message =
        bincode::deserialize(&serialized_message).expect("Deserialization error");
    let plaintext = decrypt(deserialized_encrypted_message, &mut cipher).unwrap();
    println!("Received {:?}", plaintext);
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EncryptedBytes {
    #[serde(with = "serde_bytes")]
    nonce: Vec<u8>,
    #[serde(with = "serde_bytes")]
    cipher_bytes: Vec<u8>,
}

fn encrypt<S: AsRef<[u8]>>(
    plain_bytes: S,
    cipher: &mut ChaCha20Poly1305,
) -> aead::Result<EncryptedBytes> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let cipher_bytes = cipher.encrypt(&nonce, plain_bytes.as_ref())?;
    Ok(EncryptedBytes {
        nonce: nonce.to_vec(),
        cipher_bytes,
    })
}

fn decrypt(
    encrypted_bytes: EncryptedBytes,
    cipher: &mut ChaCha20Poly1305,
) -> aead::Result<Vec<u8>> {
    // let nonce_new: Nonce = (&c.nonce).into_iter().cloned().collect();
    let nonce: Nonce = encrypted_bytes.nonce.into_iter().collect();
    let plain_bytes: Vec<u8> = cipher.decrypt(&nonce, encrypted_bytes.cipher_bytes.as_ref())?;
    Ok(plain_bytes)
}
