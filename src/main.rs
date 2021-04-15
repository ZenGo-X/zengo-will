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

    if args.insecure {
        eprintln!("WARN! Running in insecure mode")
    }

    let server_identity = match (args.cert, args.key) {
        (Some(cert), Some(key)) => {
            let cert = fs::read(cert).await.context("read server certificate")?;
            let key = fs::read(key).await.context("read server private key")?;
            Some(Identity::from_pem(cert, key))
        }
        _ => None,
    };

    let testator_ca = match args.testator_ca {
        Some(testator_ca) => {
            let testator_ca = fs::read(testator_ca).await.context("read testator ca")?;
            Some(Certificate::from_pem(testator_ca))
        }
        None => None,
    };

    let beneficiary_addr = format!("0.0.0.0:{}", args.beneficiary_api_port)
        .parse()
        .context("construct beneficiary addr")?;
    let testator_addr = format!("0.0.0.0:{}", args.testator_api_port)
        .parse()
        .context("construct testator addr")?;

    let vdf_setup = if args.vdf_params.is_some()
        && args
            .vdf_params
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false)
    {
        println!("Using cached VDF params");
        let vdf_params = fs::read(
            args.vdf_params
                .as_ref()
                .expect("guaranteed by wrapping if-expression"),
        )
        .await
        .context("read vdf parameters from file")?;
        serde_json::from_slice(&vdf_params).context("parse vdf params from file")?
    } else {
        println!("Computing VDF parameters, this might take a while");
        let vdf_setup = rsa_vdf::SetupForVDF::public_setup(&args.t.into());
        println!("VDF parameters are ready");
        if let Some(vdf_path) = args.vdf_params.as_ref() {
            let vdf_params =
                serde_json::to_vec(&vdf_setup).context("serialize vdf setup params")?;
            fs::write(vdf_path, vdf_params)
                .await
                .context("save vdf setup params to file")?
        }
        vdf_setup
    };

    if let Some(dir) = args.persistent_store.parent() {
        fs::create_dir_all(dir)
            .await
            .context("create parent dir for persistent store")?
    }

    let store = SledDB::<GE>::open(args.persistent_store)
        .await
        .context("open persistent store")?;
    let beneficiary_server = server::BeneficiaryServer::new(vdf_setup, store.clone());
    let testator_server = server::TestatorServer::new(store);

    let mut beneficiary_server_builder = match server_identity.clone() {
        Some(server_identity) => Server::builder()
            .tls_config(ServerTlsConfig::new().identity(server_identity.clone()))
            .context("set TLS config")?,
        None => Server::builder(),
    };
    let beneficiary_server = beneficiary_server_builder
        .add_service(BeneficiaryApiServer::new(beneficiary_server))
        .serve(beneficiary_addr)
        .fuse();

    let mut testator_server_builder = match (server_identity, testator_ca) {
        (Some(server_identity), Some(testator_ca)) => Server::builder()
            .tls_config(
                ServerTlsConfig::new()
                    .identity(server_identity)
                    .client_ca_root(testator_ca),
            )
            .context("set TLS config")?,
        _ => Server::builder(),
    };
    let testator_server = testator_server_builder
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
