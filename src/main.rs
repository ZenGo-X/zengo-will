use anyhow::{Context, Result};

use futures::future::FutureExt;
use tokio::fs;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

use structopt::StructOpt;

use curv::elliptic::curves::secp256_k1::GE;

use crate::persistent_store::{sled::SledDB, PersistentStore};
use crate::proto::{
    beneficiary::beneficiary_api_server::BeneficiaryApiServer,
    testator::testator_api_server::TestatorApiServer,
};

mod cli;
mod persistent_store;
mod proto;
mod sealed;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: cli::App = StructOpt::from_args();

    let cert = fs::read(args.cert)
        .await
        .context("read server certificate")?;
    let key = fs::read(args.key)
        .await
        .context("read server private key")?;
    let testator_ca = fs::read(args.testator_ca)
        .await
        .context("read testator ca")?;

    let server_identity = Identity::from_pem(cert, key);
    let testator_ca = Certificate::from_pem(testator_ca);

    let beneficiary_addr = format!("0.0.0.0:{}", args.beneficiary_api_port)
        .parse()
        .context("construct beneficiary addr")?;
    let testator_addr = format!("0.0.0.0:{}", args.testator_api_port)
        .parse()
        .context("construct testator addr")?;

    let vdf_setup = rsa_vdf::SetupForVDF::public_setup(&args.t.into());

    let store = SledDB::<GE>::open(args.persistent_store)
        .await
        .context("open persistent store")?;
    let beneficiary_server = server::BeneficiaryServer::new(vdf_setup, store.clone());
    let testator_server = server::TestatorServer::new(store);

    let beneficiary_server = Server::builder()
        .tls_config(ServerTlsConfig::new().identity(server_identity.clone()))
        .context("set TLS config")?
        .add_service(BeneficiaryApiServer::new(beneficiary_server))
        .serve(beneficiary_addr)
        .fuse();

    let testator_server = Server::builder()
        .tls_config(
            ServerTlsConfig::new()
                .identity(server_identity)
                .client_ca_root(testator_ca),
        )
        .context("set TLS config")?
        .add_service(TestatorApiServer::new(testator_server))
        .serve(testator_addr)
        .fuse();

    let ctrl_c = tokio::signal::ctrl_c().fuse();

    futures::pin_mut!(beneficiary_server);
    futures::pin_mut!(testator_server);
    futures::pin_mut!(ctrl_c);

    println!("Server started. Use Ctrl-C to exit.");
    for _ in 0u8..2 {
        let (which_server, result): (&str, Result<(), tonic::transport::Error>) = futures::select! {
            result = beneficiary_server => ("beneficiary", result),
            result = testator_server => ("testator", result),
            _ = ctrl_c => {
                eprintln!("Execution terminated by Ctrl-C");
                break
            }
        };
        eprintln!("{} server terminated: {:?}", which_server, result)
    }

    Ok(())
}
