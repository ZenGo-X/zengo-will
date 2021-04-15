use anyhow::{anyhow, bail, Context};
use structopt::StructOpt;

use tonic::transport::Channel;
use tonic::Request;

use curv::arithmetic::Converter;
use curv::elliptic::curves::secp256_k1::{FE, GE};
use curv::elliptic::curves::traits::{ECPoint, ECScalar};
use curv::BigInt;

use proto::beneficiary::beneficiary_api_client::BeneficiaryApiClient;
use proto::testator::testator_api_client::TestatorApiClient;

mod cli;
mod proto;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: cli::App = StructOpt::from_args();
    match args {
        cli::App::GenShare => emulate_keygen().await,
        cli::App::Testator(cli::TestatorCmd::SaveShare(args)) => testator_save_share(args).await,
        cli::App::Beneficiary(cli::BeneficiaryCmd::Verify(args)) => {
            beneficiary_verify_share(args).await
        }
        _ => todo!(),
    }
}

async fn emulate_keygen() -> anyhow::Result<()> {
    let testator_secret = FE::new_random();
    let beneficiary_secret = FE::new_random();
    let joint_pk = GE::generator() * testator_secret.clone() * beneficiary_secret.clone();

    let testator_secret = testator_secret.to_big_int().to_hex();
    let beneficiary_secret = beneficiary_secret.to_big_int().to_hex();
    let joint_pk = hex::encode(&joint_pk.pk_to_key_slice()[1..]);

    println!(
        "Beneficiary's share: {}\n\
         Testator's share:    {}\n\
         Public key:          {}",
        beneficiary_secret, testator_secret, joint_pk
    );

    Ok(())
}

async fn connect_to_beneficiary_api(
    endpoint: cli::BeneficiaryServer,
) -> anyhow::Result<BeneficiaryApiClient<Channel>> {
    let channel = Channel::from_shared(endpoint.address)
        .context("invalid beneficiary url")?
        .connect()
        .await
        .context("connect to beneficiary server")?;
    Ok(BeneficiaryApiClient::new(channel))
}

async fn connect_to_testator_api(
    endpoint: cli::TestatorServer,
) -> anyhow::Result<TestatorApiClient<Channel>> {
    let channel = Channel::from_shared(endpoint.address)
        .context("invalid beneficiary url")?
        .connect()
        .await
        .context("connect to beneficiary server")?;
    Ok(TestatorApiClient::new(channel))
}

async fn testator_save_share(args: cli::TestatorSaveShare) -> anyhow::Result<()> {
    let mut server = connect_to_testator_api(args.will_server).await?;

    server
        .save_server_share(Request::new(proto::testator::SaveServerShareRequest {
            public_key: args.public_key,
            server_secret_share: args.secret_share,
        }))
        .await
        .context("sending save share request")?;

    println!("Secret share saved");

    Ok(())
}

async fn beneficiary_verify_share(args: cli::BeneficiaryVerify) -> anyhow::Result<()> {
    let mut server = connect_to_beneficiary_api(args.will_server).await?;

    let public_key_point =
        GE::from_bytes(&args.public_key).map_err(|_| anyhow!("invalid public key"))?;

    let client_secret_share: FE = ECScalar::from(&BigInt::from_bytes(&args.secret_share));
    let client_public_share: GE = GE::generator() * client_secret_share;
    let client_public_share_slice = &client_public_share.pk_to_key_slice()[1..];

    let response = server
        .verify_server_share(Request::new(proto::beneficiary::VerifyServerShareRequest {
            public_key: args.public_key,
            client_public_share: client_public_share_slice.into(),
        }))
        .await
        .context("sending verify share request")?
        .into_inner();

    let server_public_share: GE = GE::from_bytes(&response.server_public_share[1..])
        .map_err(|_e| anyhow!("server provided invalid proof"))?;
    if server_public_share * client_secret_share == public_key_point {
        println!("Server proofed that it owns a valid share");
    } else {
        bail!("Server provided incorrect proof!");
    }
    Ok(())
}
