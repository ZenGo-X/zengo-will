# ZenGo Will

## Demo

1. Start the Will server
   ```bash
   cargo run -- -t 10 --insecure --persistent-store target/tmp-store
   ```

2. Emulate keygen
   ```bash
   cargo run --example demo -- gen-share
   ```
   
   This will output server and client shares:
   ```text
   Beneficiary's share: TBD_CS
   Testator's share:    TBD_SS
   Public key:          TBD_PK
   ```
   
3. Testator sends to Will its share
   ```bash
   cargo run --example demo -- testator save-share --secret-share TBD_SS --public-key TBD_PK
   ```
   
4. Beneficiary verifies that Will received a share
   ```bash
   cargo run --example demo -- beneficiary verify --secret-share TBD_CS --public-key TBD_PK
   ```

5. Testator sends keepalive messages to Will:
   ```bash
   cargo run --example demo -- testator send-keepalive --every 1s
   ```
   
6. Beneficiary tries to obtain testator share, but unsuccessfully as testator sends keepalive messages:
   ```bash
   cargo run --example demo -- beneficiary claim --secret-share TBD_CS --public-key TBD_PK
   ```
   
   It will output:
   ```text
   Failed: Testator is alive
   ```

7. Kill testator by sending Ctrl-C to the terminal from step 5. Now beneficiary is able to claim a counter-party's
   secret share:
   ```bash
   cargo run --example demo -- beneficiary claim --secret-share TBD_CS --public-key TBD_PK
   ```

   Outputs:
   ```text
   Obtained testator secret: TBD_SS
   ```
