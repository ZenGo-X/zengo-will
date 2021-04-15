use structopt::StructOpt;

use curv::arithmetic::Converter;
use curv::elliptic::curves::secp256_k1::{FE, GE};
use curv::elliptic::curves::traits::{ECPoint, ECScalar};

mod cli;
mod proto;

#[tokio::main]
async fn main() {
    let args: cli::App = StructOpt::from_args();
    match args {
        cli::App::GenShare => emulate_keygen().await,
        _ => todo!(),
    }
}

async fn emulate_keygen() {
    let testator_secret = FE::new_random();
    let beneficiary_secret = FE::new_random();
    let joint_pk = GE::generator() * testator_secret.clone() * beneficiary_secret.clone();

    let testator_secret = testator_secret.to_big_int().to_hex();
    let beneficiary_secret = beneficiary_secret.to_big_int().to_hex();
    let joint_pk = hex::encode(joint_pk.pk_to_key_slice());

    println!(
        "Beneficiary's share: {}\n\
         Testator's share:    {}\n\
         Public key:          {}",
        beneficiary_secret, testator_secret, joint_pk
    );
}
