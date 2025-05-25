# 🕵️ Block Explorer (Optional CLI)

Reads on-disk DB to allow inspection of:

## 🧾 Commands

- `explorer list-blocks`
- `explorer get-block <hash>`
- `explorer get-tx <hash>`
- `explorer get-balance <address>`

## 🧰 Data Source

- Reads from RocksDB directly
- No network access required

## 👁️ Output

- Human-readable terminal output
- Optional: JSON flag