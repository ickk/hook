//! Provides `hmac_sha256` function to compute a SHA256 HMAC.

use sha2::{Sha256,
           Digest};

/// Compute the SHA-256 HMAC of `message` using `key`.
pub fn hmac_sha256(key: Vec<u8>, message: Vec<u8>) -> Vec<u8>{
  const BLOCK_SIZE: usize = 512/8;
  let mut message = message.clone();

  // Normalise the key
  let mut key_norm: Vec<u8> = vec![0 as u8; BLOCK_SIZE];
  match key.len() > BLOCK_SIZE {
    true => {
      sum(&mut key_norm, sha256_digest(&key))
    },
    false => {
      sum(&mut key_norm, key)
    },
  };

  // Define the inner and outer padding
  let mut outer_padding = xor(&mut key_norm.clone(),
                              vec![0x5c as u8; BLOCK_SIZE]).to_owned();
  let mut inner_padding = xor(&mut key_norm.clone(),
                              vec![0x36 as u8; BLOCK_SIZE]).to_owned();

  // Compute the MAC
  sha256_digest({
    outer_padding.append(&mut
      sha256_digest({
        inner_padding.append(&mut message);
        &inner_padding}));
    &outer_padding})
}

/// Computes the SHA-256 digest of `message`.
#[inline]
fn sha256_digest(message: &Vec<u8>) -> Vec<u8> {
  let mut hasher = Sha256::new();
  hasher.update(message);
  hasher.finalize().to_vec()
}

/// Performs element-wise summation of two `Vec<u8>`s.
/// `vec_a` is mutated to store the result.
#[inline]
fn sum(vec_a: &mut Vec<u8>, vec_b: Vec<u8>) -> &mut Vec<u8> {
  for (a, b) in vec_a.iter_mut().zip(vec_b.iter()) {
    *a = *a + b;
  }
  vec_a
}

/// Performs bit-wise XOR on two `Vec<u8>`s.
/// `vec_a` is mutated to store the result.
#[inline]
fn xor(vec_a: &mut Vec<u8>, vec_b: Vec<u8>) -> &mut Vec<u8> {
  for (a, b) in vec_a.iter_mut().zip(vec_b.iter()) {
    *a = *a ^ b;
  }
  vec_a
}
