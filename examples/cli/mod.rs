use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;

type Hex = Vec<u8>;

#[derive(StructOpt, Debug)]
pub enum App {
    Testator(TestatorCmd),
    Beneficiary(BeneficiaryCmd),
    GenShare,
}

#[derive(StructOpt, Debug)]
pub enum TestatorCmd {
    SaveShare(TestatorSaveShare),
    SendKeepalive(TestatorSendKeepalive),
}

#[derive(StructOpt, Debug)]
pub enum BeneficiaryCmd {
    Verify(BeneficiaryVerify),
    Claim(BeneficiaryClaim),
}

#[derive(StructOpt, Debug)]
pub struct TestatorSaveShare {
    #[structopt(long, parse(try_from_str = hex::decode))]
    pub secret_share: Hex,
    #[structopt(long, parse(try_from_str = hex::decode))]
    pub public_key: Hex,

    #[structopt(flatten)]
    pub will_server: TestatorServer,
}

#[derive(StructOpt, Debug)]
pub struct TestatorSendKeepalive {
    #[structopt(long, parse(try_from_str = parse_duration::parse))]
    pub every: Duration,

    #[structopt(flatten)]
    pub will_server: TestatorServer,
}

#[derive(StructOpt, Debug)]
pub struct BeneficiaryVerify {
    #[structopt(long, parse(try_from_str = hex::decode))]
    pub secret_share: Hex,
    #[structopt(long, parse(try_from_str = hex::decode))]
    pub public_key: Hex,

    #[structopt(flatten)]
    pub will_server: BeneficiaryServer,
}

#[derive(StructOpt, Debug)]
pub struct BeneficiaryClaim {
    #[structopt(long, parse(try_from_str = hex::decode))]
    pub secret_share: Hex,
    #[structopt(long, parse(try_from_str = hex::decode))]
    pub public_key: Hex,

    #[structopt(flatten)]
    pub will_server: BeneficiaryServer,
}

#[derive(StructOpt, Debug)]
pub struct BeneficiaryServer {
    #[structopt(long, default_value = "http://127.0.0.1:4949")]
    pub address: String,
    #[structopt(long)]
    pub will_cert: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct TestatorServer {
    #[structopt(long, default_value = "http://127.0.0.1:4950")]
    pub address: String,
    #[structopt(long, requires_all(&["my_cert", "my_key"]))]
    pub will_cert: Option<PathBuf>,
    #[structopt(long, requires_all(&["will_cert", "my_key"]))]
    pub my_cert: Option<PathBuf>,
    #[structopt(long, requires_all(&["will_cert", "my_cert"]))]
    pub my_key: Option<PathBuf>,
}
