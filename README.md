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
   Beneficiary's share: c4834b14beea22181406396c09a380a5ff8c0e579ed0ab8fe60473f6f119931e
   Testator's share:    adff4b84bfabdc6979fe306719247a8d61ea5fe1f2fa36f6e7ef85f2e4592146
   Public key:          94fcb56210eae5d57ea0f3dcf3fba2b92a33ed92cccbd0b960e04e3fc8ee9dcdbd366492ee3c1b67849c76a93b5ecf59458302627bff1db670a386fa21b86008
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
   
   Error message will be printed in the terminal saying testator is alive.

7. Kill testator by sending Ctrl-C to the terminal from step 5. Now beneficiary is able to claim a counter-party's
   secret share:
   ```bash
   cargo run --example demo -- beneficiary claim --secret-share TBD_CS --public-key TBD_PK
   ```

   Outputs:
   ```text
   Retrieving challenge from the server
   Solving challenge
   Challenge solved. Sending it to server
   Testator secret share: adff4b84bfabdc6979fe306719247a8d61ea5fe1f2fa36f6e7ef85f2e4592146
   ```
