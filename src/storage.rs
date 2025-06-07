use crate::block::{Block, BlockHeader};
use crate::state_machine::{Account, WorldState};
use crate::types::{Address, Hash, BlockHeight};
use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use thiserror::Error;

const BLOCKS_CF: &str = "blocks";
const HEADERS_CF: &str = "headers";
const STATE_CF: &str = "state";
const META_CF: &str = "meta";

const TIP_KEY: &[u8] = b"tip";
const HEIGHT_KEY: &[u8] = b"height";

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("RocksDB error: {0}")]
    DbError(#[from] rocksdb::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Item not found: {0}")]
    NotFound(String),
}

pub struct Storage {
    db: DB,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let cfs = [BLOCKS_CF, HEADERS_CF, STATE_CF, META_CF];
        let db = DB::open_cf(&opts, path, cfs)?;
        
        Ok(Storage { db })
    }

    fn get_cf(&self, cf_name: &str) -> Result<&rocksdb::ColumnFamily, StorageError> {
        self.db.cf_handle(cf_name).ok_or_else(|| StorageError::NotFound(format!("Column family '{}' not found", cf_name)))
    }

    pub fn get_block(&self, hash: &Hash) -> Result<Option<Block>, StorageError> {
        let cf = self.get_cf(BLOCKS_CF)?;
        let result = self.db.get_cf(cf, hash.0)?;
        result.map(|bytes| bincode::decode_from_slice(&bytes, bincode::config::standard()).map(|(block, _)| block).map_err(|e| StorageError::DeserializationError(e.to_string()))).transpose()
    }

    pub fn put_block(&self, block: &Block) -> Result<(), StorageError> {
        let cf = self.get_cf(BLOCKS_CF)?;
        let hash = block.header.calculate_hash().map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let bytes = bincode::encode_to_vec(block, bincode::config::standard()).map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, hash.0, bytes)?;
        Ok(())
    }

    pub fn get_account(&self, address: &Address) -> Result<Option<Account>, StorageError> {
        let cf = self.get_cf(STATE_CF)?;
        let result = self.db.get_cf(cf, address.0)?;
        result.map(|bytes| bincode::decode_from_slice(&bytes, bincode::config::standard()).map(|(account, _)| account).map_err(|e| StorageError::DeserializationError(e.to_string()))).transpose()
    }

    pub fn put_account(&self, address: &Address, account: &Account) -> Result<(), StorageError> {
        let cf = self.get_cf(STATE_CF)?;
        let bytes = bincode::encode_to_vec(account, bincode::config::standard()).map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, address.0, bytes)?;
        Ok(())
    }

    pub fn get_tip(&self) -> Result<Option<Hash>, StorageError> {
        let cf = self.get_cf(META_CF)?;
        let result = self.db.get_cf(cf, TIP_KEY)?;
        result.map(|bytes| bincode::decode_from_slice(&bytes, bincode::config::standard()).map(|(hash, _)| hash).map_err(|e| StorageError::DeserializationError(e.to_string()))).transpose()
    }

    pub fn put_tip(&self, hash: &Hash) -> Result<(), StorageError> {
        let cf = self.get_cf(META_CF)?;
        let bytes = bincode::encode_to_vec(hash, bincode::config::standard()).map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, TIP_KEY, bytes)?;
        Ok(())
    }

    pub fn get_chain_tip(&self) -> Result<Option<(Hash, u64)>, StorageError> {
        let cf = self.get_cf(META_CF)?;
        
        // Get the tip hash
        let tip_result = self.db.get_cf(cf, TIP_KEY)?;
        let tip_hash = match tip_result {
            Some(bytes) => bincode::decode_from_slice(&bytes, bincode::config::standard())
                .map(|(hash, _)| hash)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?,
            None => return Ok(None),
        };
        
        // Get the height
        let height_result = self.db.get_cf(cf, HEIGHT_KEY)?;
        let height = match height_result {
            Some(bytes) => bincode::decode_from_slice::<u64, _>(&bytes, bincode::config::standard())
                .map(|(height, _)| height)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?,
            None => return Ok(None),
        };
        
        Ok(Some((tip_hash, height)))
    }

    pub fn set_chain_tip(&self, hash: &Hash, height: u64) -> Result<(), StorageError> {
        let cf = self.get_cf(META_CF)?;
        
        let hash_bytes = bincode::encode_to_vec(hash, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, TIP_KEY, hash_bytes)?;
        
        let height_bytes = bincode::encode_to_vec(&height, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, HEIGHT_KEY, height_bytes)?;
        
        Ok(())
    }

    pub fn put_header_by_height(&self, height: u64, header: &BlockHeader) -> Result<(), StorageError> {
        let cf = self.get_cf(HEADERS_CF)?;
        let key = height.to_be_bytes(); // Use big-endian encoding for consistent sorting
        let bytes = bincode::encode_to_vec(header, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, key, bytes)?;
        Ok(())
    }
    
    pub fn commit_block(&self, block: &Block, world_state: &WorldState) -> Result<(), StorageError> {
        let mut batch = WriteBatch::default();
        let block_cf = self.get_cf(BLOCKS_CF)?;
        let state_cf = self.get_cf(STATE_CF)?;
        let meta_cf = self.get_cf(META_CF)?;

        let hash = block.header.calculate_hash().map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let block_bytes = bincode::encode_to_vec(block, bincode::config::standard()).map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&block_cf, hash.0, block_bytes);

        for (address, account) in world_state {
            let account_bytes = bincode::encode_to_vec(account, bincode::config::standard()).map_err(|e| StorageError::SerializationError(e.to_string()))?;
            batch.put_cf(&state_cf, address.0, account_bytes);
        }

        // Store both tip hash and height
        let tip_bytes = bincode::encode_to_vec(&hash, bincode::config::standard()).map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&meta_cf, TIP_KEY, tip_bytes);
        
        let height_bytes = bincode::encode_to_vec(&block.header.block_number.0, bincode::config::standard()).map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&meta_cf, HEIGHT_KEY, height_bytes);

        self.db.write(batch)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockHeader;
    use crate::types::{Address, Nonce, Signature};
    use tempfile::tempdir;
    
    fn temp_db_path() -> tempfile::TempDir {
        tempdir().unwrap()
    }

    #[test]
    fn test_put_and_get_block() {
        let db_path = temp_db_path();
        let storage = Storage::new(db_path.path()).unwrap();
        let block = Block {
            header: BlockHeader {
                parent_hash: Hash([0; 32]),
                block_number: BlockHeight(1),
                timestamp: crate::types::Timestamp(123),
                tx_root: Hash([1; 32]),
                validator: Address([2; 32]),
                signature: Signature(ed25519_dalek::Signature::from_bytes(&[0; 64]).to_bytes().to_vec()),
            },
            transactions: vec![],
        };
        let hash = block.header.calculate_hash().unwrap();

        storage.put_block(&block).unwrap();
        let retrieved_block = storage.get_block(&hash).unwrap().unwrap();
        assert_eq!(block, retrieved_block);
    }

    #[test]
    fn test_put_and_get_account() {
        let db_path = temp_db_path();
        let storage = Storage::new(db_path.path()).unwrap();
        let address = Address([1; 32]);
        let account = Account {
            balance: 100,
            nonce: Nonce(1),
        };

        storage.put_account(&address, &account).unwrap();
        let retrieved_account = storage.get_account(&address).unwrap().unwrap();
        assert_eq!(account, retrieved_account);
    }

    #[test]
    fn test_put_and_get_tip() {
        let db_path = temp_db_path();
        let storage = Storage::new(db_path.path()).unwrap();
        let tip_hash = Hash([1; 32]);

        storage.put_tip(&tip_hash).unwrap();
        let retrieved_tip = storage.get_tip().unwrap().unwrap();
        assert_eq!(tip_hash, retrieved_tip);
    }

    #[test]
    fn test_commit_block() {
        let db_path = temp_db_path();
        let storage = Storage::new(db_path.path()).unwrap();
        let address = Address([1; 32]);
        let account = Account {
            balance: 100,
            nonce: Nonce(1),
        };
        let mut world_state = WorldState::new();
        world_state.insert(address, account);
        let block = Block {
            header: BlockHeader {
                parent_hash: Hash([0; 32]),
                block_number: BlockHeight(1),
                timestamp: crate::types::Timestamp(123),
                tx_root: Hash([1; 32]),
                validator: Address([2; 32]),
                signature: Signature(ed25519_dalek::Signature::from_bytes(&[0; 64]).to_bytes().to_vec()),
            },
            transactions: vec![],
        };
        let hash = block.header.calculate_hash().unwrap();

        storage.commit_block(&block, &world_state).unwrap();
        
        let retrieved_block = storage.get_block(&hash).unwrap().unwrap();
        assert_eq!(block, retrieved_block);

        let retrieved_account = storage.get_account(&address).unwrap().unwrap();
        assert_eq!(world_state.get(&address).unwrap(), &retrieved_account);

        let retrieved_tip = storage.get_tip().unwrap().unwrap();
        assert_eq!(hash, retrieved_tip);
    }
}
