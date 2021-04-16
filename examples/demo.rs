use anyhow::{anyhow, bail, Context};
use structopt::StructOpt;

use tokio::fs;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
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
    tracing_subscriber::fmt::init();
    let args: cli::App = StructOpt::from_args();
    match args {
        cli::App::GenShare => emulate_keygen().await,
        cli::App::Testator(cli::TestatorCmd::SaveShare(args)) => testator_save_share(args).await,
        cli::App::Testator(cli::TestatorCmd::SendKeepalive(args)) => {
            testator_send_keepalive(args).await
        }
        cli::App::Beneficiary(cli::BeneficiaryCmd::Verify(args)) => {
            beneficiary_verify_share(args).await
        }
        cli::App::Beneficiary(cli::BeneficiaryCmd::Claim(args)) => beneficiary_claim(args).await,
    }
}

async fn emulate_keygen() -> anyhow::Result<()> {
    let testator_secret = FE::new_random();
    let beneficiary_secret = FE::new_random();
    let joint_pk = GE::generator() * testator_secret.clone() * beneficiary_secret.clone();

    let testator_secret = add_leading_zero(testator_secret.to_big_int().to_hex());
    let beneficiary_secret = add_leading_zero(beneficiary_secret.to_big_int().to_hex());
    let joint_pk = hex::encode(&joint_pk.pk_to_key_slice()[1..]);

    println!(
        "Beneficiary's share: {}\n\
         Testator's share:    {}\n\
         Public key:          {}",
        beneficiary_secret, testator_secret, joint_pk
    );

    Ok(())
}

fn add_leading_zero(hex: String) -> String {
    if hex.len() % 2 == 1 {
        "0".to_owned() + &hex
    } else {
        hex
    }
}

async fn connect_to_beneficiary_api(
    mut endpoint: cli::BeneficiaryServer,
) -> anyhow::Result<BeneficiaryApiClient<Channel>> {
    let tls_config = if let Some(will_cert) = endpoint.will_ca {
        let cert = fs::read(will_cert)
            .await
            .context("read Will server certificate")?;
        let cert = Certificate::from_pem(cert);
        Some(ClientTlsConfig::new().ca_certificate(cert))
    } else {
        eprintln!("WARN: Connecting to Will server over insecure channel");
        endpoint.address = endpoint.address.replace("https://", "http://");
        None
    };
    let endpoint = Channel::from_shared(endpoint.address).context("invalid beneficiary url")?;
    let endpoint = match tls_config {
        Some(tls_config) => endpoint.tls_config(tls_config).context("set tls config")?,
        None => endpoint,
    };
    let channel = endpoint
        .connect()
        .await
        .context("connect to beneficiary server")?;
    Ok(BeneficiaryApiClient::new(channel))
}

async fn connect_to_testator_api(
    mut endpoint: cli::TestatorServer,
) -> anyhow::Result<TestatorApiClient<Channel>> {
    let tls_config = match (endpoint.will_ca, endpoint.cert, endpoint.key) {
        (Some(will_cert), Some(my_cert), Some(my_key)) => {
            let will_cert = fs::read(will_cert)
                .await
                .context("read Will server certificate")?;
            let my_cert = fs::read(my_cert).await.context("read my certificate")?;
            let my_key = fs::read(my_key).await.context("read my private key")?;

            let will_cert = Certificate::from_pem(will_cert);
            let my_identity = Identity::from_pem(my_cert, my_key);

            Some(
                ClientTlsConfig::new()
                    .ca_certificate(will_cert)
                    .identity(my_identity),
            )
        }
        _ => {
            eprintln!("WARN: Connecting to Will server over insecure channel");
            endpoint.address = endpoint.address.replace("https://", "http://");
            None
        }
    };
    let endpoint = Channel::from_shared(endpoint.address).context("invalid testator url")?;
    let endpoint = match tls_config {
        Some(tls_config) => endpoint.tls_config(tls_config).context("set tls config")?,
        None => endpoint,
    };
    let channel = endpoint
        .connect()
        .await
        .context("connect to testator server")?;
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

async fn testator_send_keepalive(args: cli::TestatorSendKeepalive) -> anyhow::Result<()> {
    let mut server = connect_to_testator_api(args.will_server).await?;

    for i in 1u64.. {
        server
            .ping(proto::testator::PingRequest {})
            .await
            .context(format!("sending ping {}", i))?;
        println!("Ping {} sent", i);
        tokio::time::sleep(args.every).await
    }

    bail!("testator tired (it sent {} pings!!)", u64::MAX)
}

async fn beneficiary_claim(args: cli::BeneficiaryClaim) -> anyhow::Result<()> {
    let mut server = connect_to_beneficiary_api(args.will_server).await?;

    let public_key_point: GE =
        GE::from_bytes(&args.public_key).map_err(|_e| anyhow!("invalid public key"))?;

    let client_secret_share: FE = ECScalar::from(&BigInt::from_bytes(&args.secret_share));
    let client_public_share: GE = GE::generator() * client_secret_share;
    let client_public_share_bytes = &client_public_share.pk_to_key_slice()[1..];

    eprintln!("Retrieving challenge from the server");
    let solving_challenge = server
        .get_challenge(Request::new(proto::beneficiary::GetChallengeRequest {}))
        .await
        .context("get challenge from server")?
        .into_inner();
    let challenge: rsa_vdf::UnsolvedVDF =
        serde_json::from_slice(&solving_challenge.challenge).context("parse challenge")?;
    eprintln!("Solving challenge");
    let solution = rsa_vdf::UnsolvedVDF::eval(&challenge);
    let solution = serde_json::to_vec(&solution).context("serialize solution")?;
    eprintln!("Challenge solved. Sending it to server");

    let response = server
        .obtain_server_secret_share(Request::new(
            proto::beneficiary::ObtainServerSecretShareRequest {
                public_key: args.public_key,
                client_public_share: client_public_share_bytes.into(),
                solved_challenge: Some(solving_challenge),
                solution,
            },
        ))
        .await
        .context("claiming share")?
        .into_inner();

    let server_secret_share: FE =
        ECScalar::from(&BigInt::from_bytes(&response.server_secret_share));
    if client_public_share * server_secret_share == public_key_point {
        println!(
            "Testator secret share: {}",
            hex::encode(response.server_secret_share)
        )
    } else {
        bail!("server sent incorrect testator's share")
    }

    Ok(())
}
