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
       --testator-ca client_ca.pem \
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

## Demo: Azure SGX machine + Anjuna runtime

### Setup

1. You need SGX machine with Anjuna installed (v0.25.0007).
   Build Will server to `x86_64-unknown-linux-gnu` target. If you're using linux, just build with cargo:
   ```bash
   cargo build --release --bin zengo-will
   ```

   If you're using Mac with Intel chip, [cross tool](https://github.com/rust-embedded/cross) is your choice:
   ```bash
   cross build --release --bin zengo-will --target x86_64-unknown-linux-gnu
   ```

   Then transfer `zengo-will` binary, [manifest.template.yaml](./manifest.template.yaml) config file, and
   [client_ca.pem](examples/data/client_ca.pem) to the SGX machine. Put them to the same directory.
   
   SGX machine should have TCP ports 4949 and 4950 opened.
   
1. Build Will demo client
   ```bash
   cargo build --example demo --release
   ```
   
   Create new directory `demo` and copy demo binary into it:
   ```bash
   mkdir demo
   cp target/release/examples/demo demo
   cd demo
   ```

### Running Demo
1. Start Will server on SGX machine:
   ```bash
   anjuna-sgxrun ./zengo-will
   ```
   
   'Will' will generate self-signed certificate, and run beneficiary and testator servers on 4949 and 4950 ports.

   We denote Will's IP address as $ADDR.

1. At your host machine, you should have demo binary compiled. First, we need to retrieve Will's certificate:
   ```bash
   ./demo get-cert --address "$ADDR:4949" --hostname will.zengo.com > server.pem
   ```

1. Then we emulate keygen between beneficiary and testator:
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
   ./demo testator save-share \
       --cert ../examples/data/client1.pem --key ../examples/data/client1.key \
       --address "https://$ADDR:4950/" --will-ca server.pem --hostname will.zengo.com \
       --public-key $PK --secret-share $TS
   ```

1. Beneficiary verifies that Will received a share
   ```bash
   ./demo beneficiary verify --address "https://$ADDR:4949/" \
       --will-ca server.pem --hostname will.zengo.com \
       --secret-share $BS --public-key $PK
   ```

   On success, demo app outputs:
   ```text
   Server proofed that it owns a valid share
   ```

1. Testator starts sending keepalive messages to Will:
   ```bash
   ./demo testator send-keepalive \
       --cert ../examples/data/client1.pem --key ../examples/data/client1.key \
       --address "https://$ADDR:4950/" --will-ca server.pem --hostname will.zengo.com \
       --every 1s
   ```

1. Beneficiary tries to obtain testator share, but unsuccessfully as testator sends keepalive messages:
   ```bash
   ./demo beneficiary claim --address "https://$ADDR:4949/" \
       --will-ca server.pem --hostname will.zengo.com \
       --secret-share $BS --public-key $PK
   ```

   Error message will be printed in the terminal saying testator is alive.

1. Kill testator by sending Ctrl-C to the terminal from step 5. Now beneficiary is able to claim a counter-party's
   secret share:
   ```bash
   ./demo beneficiary claim --address "https://$ADDR:4949/" \
       --will-ca server.pem --hostname will.zengo.com \
       --secret-share $BS --public-key $PK
   ```

   Outputs:
   ```text
   Retrieving challenge from the server
   Solving challenge
   Challenge solved. Sending it to server
   Testator secret share: adff4b84bfabdc6979fe306719247a8d61ea5fe1f2fa36f6e7ef85f2e4592146
   ```
