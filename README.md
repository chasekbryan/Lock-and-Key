```text
==============
 LOCK & KEY - v0.1
==============
```

1. Build:
   ```bash
   cargo build --release
   ```
2. Encrypt:
   ```bash
   ./target/release/lock path/to/yourfile.txt
   ```
   **Output:**
   ```text
   Enter passphrase: ******
   Encrypted to: yourfile.lock
   ```
3. Decrypt 'key' is run in the same manner as encryption
