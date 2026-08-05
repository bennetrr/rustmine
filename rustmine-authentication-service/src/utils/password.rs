use crate::utils::error::Error;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// Generates the hash from the given password
pub(crate) fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| Error::Unexpected(e.to_string()))
}

/// Validates the password against the hash
pub(crate) fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .and_then(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password_success() {
        let password = "SuperSecretPassword123!";

        let hash_result = hash_password(password);
        assert!(hash_result.is_ok(), "Password hashing failed unexpectedly");

        let hash = hash_result.unwrap();

        assert_ne!(hash, password);
        assert!(hash.starts_with("$argon2"));

        let is_valid = verify_password(password, &hash);
        assert!(
            is_valid,
            "Password verification failed with the correct password"
        );
    }

    // negative test
    #[test]
    fn test_verify_password_fails_with_wrong_password() {
        let correct_password = "MyCorrectPassword123";
        let wrong_password = "MyWrongPassword123";

        let hash = hash_password(correct_password).unwrap();

        let is_valid = verify_password(wrong_password, &hash);
        assert!(
            !is_valid,
            "Verification succeeded despite using a wrong password"
        );
    }

    #[test]
    fn test_verify_password_fails_with_malformed_hash() {
        let password = "AnyPassword123";
        let invalid_hash = "$argon2id$v=19$m=16,t=2,p=1$invalidhashstring";

        let is_valid = verify_password(password, invalid_hash);
        assert!(
            !is_valid,
            "Verification should fail on a corrupted or malformed hash string"
        );

        let is_valid_garbage = verify_password(password, "completely_random_garbage");
        assert!(
            !is_valid_garbage,
            "Verification should fail on absolute non-hash string"
        );
    }

    #[test]
    fn test_salting_ensures_unique_hashes() {
        let password = "same_password_for_both";

        let hash_a = hash_password(password).unwrap();
        let hash_b = hash_password(password).unwrap();

        assert_ne!(
            hash_a, hash_b,
            "Salting is not unique! Both hashes are identical."
        );
    }
}
