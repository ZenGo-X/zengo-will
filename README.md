# ZenGo Will

## Demo

0. Build the Will server and demo app:
   ```bash
   cargo build --release --example demo --bin zengo-will
   mkdir demo
   cp target/release/zengo-will target/release/examples/demo demo
   cd demo
   ```

1. Start the Will server
   ```bash
   ./zengo-will --generate-self-signed will.zengo.com \
       --testator-ca ../examples/data/client_ca.pem \
       -t 100000 --persistent-store store/ --vdf-params vdf-params.json
   ```

1. Retrieve Will server certificate:
   ```bash
   ./demo get-cert --address 127.0.0.1:4949 --hostname will.zengo.com > server.pem
   ```

1. Emulate keygen
   ```bash
   ./demo gen-share
   ```
   
   This will output testator's and beneficiary's shares:
   ```text
   Beneficiary's share: c4834b14beea22181406396c09a380a5ff8c0e579ed0ab8fe60473f6f119931e
   Testator's share:    adff4b84bfabdc6979fe306719247a8d61ea5fe1f2fa36f6e7ef85f2e4592146
   Public key:          94fcb56210eae5d57ea0f3dcf3fba2b92a33ed92cccbd0b960e04e3fc8ee9dcdbd366492ee3c1b67849c76a93b5ecf59458302627bff1db670a386fa21b86008
   ```
   
   We denote Beneficiary's secret share as $BS, Testator's secret share as $TS, and their joint public key as $PK.
   
1. Testator sends to Will its share
   ```bash
   ./demo testator save-share --cert ../examples/data/client1.pem --key ../examples/data/client1.key \
       --will-ca server.pem --hostname will.zengo.com \
       --public-key $PK --secret-share $TS
   ```
   
1. Beneficiary verifies that Will received a share
   ```bash
   ./demo beneficiary verify --will-ca server.pem --hostname will.zengo.com \
       --secret-share $BS --public-key $PK
   ```
   
   On success, demo app outputs:
   ```text
   Server proofed that it owns a valid share
   ```

1. Testator starts sending keepalive messages to Will:
   ```bash
   ./demo testator send-keepalive --cert ../examples/data/client1.pem --key ../examples/data/client1.key \
       --will-ca server.pem --hostname will.zengo.com \
       --every 1s
   ```
   
1. Beneficiary tries to obtain testator share, but unsuccessfully as testator sends keepalive messages:
   ```bash
   ./demo beneficiary claim --will-ca server.pem --hostname will.zengo.com \
       --secret-share $BS --public-key $PK
   ```
   
   Error message will be printed in the terminal saying testator is alive.

1. Kill testator by sending Ctrl-C to the terminal from step 5. Now beneficiary is able to claim a counter-party's
   secret share:
   ```bash
   ./demo beneficiary claim --will-ca server.pem --hostname will.zengo.com \
       --secret-share $BS --public-key $PK
   ```

   Outputs:
   ```text
   Retrieving challenge from the server
   Solving challenge
   Challenge solved. Sending it to server
   Testator secret share: adff4b84bfabdc6979fe306719247a8d61ea5fe1f2fa36f6e7ef85f2e4592146
   ```
