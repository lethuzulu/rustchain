## Wallet Library Interface

**Purpose:** Defines a standard library interface for core wallet operations such as cryptographic key management, address derivation, and transaction creation/signing. This interface is intended for use by CLI tools, client applications, or testing utilities, separate from the node's RPC interface.

**Public Functions/Methods:**

*   **`generate_keypair() -> Result<(PrivateKey, PublicKey), WalletError>`**
    *   **Description:** Generates a new cryptographic key pair (private and public keys) suitable for creating an account and signing transactions.
    *   **Parameters:** None.
    *   **Returns:** A tuple containing the `PrivateKey` and `PublicKey`, or a `WalletError` if generation fails.

*   **`load_keypair_from_file(path: &Path) -> Result<(PrivateKey, PublicKey), WalletError>`**
    *   **Description:** Loads a previously saved key pair from a specified file path.
    *   **Parameters:**
        *   `path`: The file system path to the key pair file.
    *   **Returns:** The loaded `PrivateKey` and `PublicKey`, or `WalletError` if loading fails (e.g., file not found, invalid format, permission issues).
    *   **Preconditions:** The file at `path` should exist and contain a validly formatted key pair.

*   **`save_keypair_to_file(path: &Path, private_key: &PrivateKey, public_key: &PublicKey) -> Result<(), WalletError>`**
    *   **Description:** Saves a given key pair to a specified file path.
    *   **Parameters:**
        *   `path`: The file system path where the key pair will be saved.
        *   `private_key`: The private key to save.
        *   `public_key`: The public key to save.
    *   **Returns:** `Ok(())` on successful save, or `WalletError` if saving fails (e.g., permission issues, disk full).
    *   **Postconditions:** The key pair is persisted to the specified file.

*   **`get_address_from_public_key(public_key: &PublicKey) -> Address`**
    *   **Description:** Derives a blockchain address from a given public key.
    *   **Parameters:**
        *   `public_key`: The public key from which to derive the address.
    *   **Returns:** The derived `Address`.

*   **`create_signed_transaction(sender_address: Address, recipient_address: Address, amount: u64, nonce: u64, private_key: &PrivateKey) -> Result<Transaction, WalletError>`**
    *   **Description:** Creates a new transaction with the given parameters and signs it using the provided private key.
    *   **Parameters:**
        *   `sender_address`: The address of the transaction sender.
        *   `recipient_address`: The address of the transaction recipient.
        *   `amount`: The amount of tokens to transfer.
        *   `nonce`: The sender's account nonce for this transaction.
        *   `private_key`: The private key of the sender, used for signing.
    *   **Returns:** The signed `Transaction` object, or `WalletError` if creation or signing fails.
    *   **Preconditions:** The private key must correspond to the sender implicit in the transaction being built (though `sender_address` is passed for clarity and can be cross-verified).

**Data Structures:**

*   `PrivateKey` (e.g., an opaque struct wrapping the raw key bytes for Ed25519 or other scheme)
*   `PublicKey` (e.g., an opaque struct wrapping the raw key bytes)
*   `Address` (see `../architecture/data_structures.md`)
*   `Transaction` (see `../architecture/data_structures.md`)

**Error Handling:**

*   `enum WalletError {`
    *   `KeyGenerationFailed,`
    *   `FileIOError(String),`         // E.g., for path not found, permission denied
    *   `KeySerializationFailed,`
    *   `KeyDeserializationFailed,`   // E.g., invalid key format in file
    *   `SigningFailed,`
    *   `InvalidParameters(String)`
    *   `}` 