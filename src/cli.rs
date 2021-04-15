use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct App {
    #[structopt(short)]
    pub t: u64,

    #[structopt(long)]
    pub persistent_store: PathBuf,

    #[structopt(long, required_unless = "insecure")]
    pub cert: Option<PathBuf>,
    #[structopt(long, required_unless = "insecure")]
    pub key: Option<PathBuf>,

    #[structopt(long, required_unless = "insecure")]
    pub testator_ca: Option<PathBuf>,

    #[structopt(long, default_value = "4949")]
    pub beneficiary_api_port: u16,
    #[structopt(long, default_value = "4950")]
    pub testator_api_port: u16,

    #[structopt(long)]
    pub vdf_params: Option<PathBuf>,

    #[structopt(long)]
    pub insecure: bool,
}
