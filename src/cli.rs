use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct App {
    #[structopt(short)]
    pub t: u64,

    #[structopt(long)]
    pub persistent_store: PathBuf,

    #[structopt(long, conflicts_with_all(&["insecure", "generate_self_signed"]))]
    pub cert: Option<PathBuf>,
    #[structopt(long, conflicts_with_all(&["insecure", "generate_self_signed"]))]
    pub key: Option<PathBuf>,

    #[structopt(long, required_unless = "insecure")]
    pub testator_ca: Option<PathBuf>,

    #[structopt(long, default_value = "4949")]
    pub beneficiary_api_port: u16,
    #[structopt(long, default_value = "4950")]
    pub testator_api_port: u16,

    #[structopt(long)]
    pub vdf_params: Option<PathBuf>,

    #[structopt(long, conflicts_with = "generate_self_singed")]
    pub insecure: bool,
    #[structopt(long)]
    pub generate_self_signed: Vec<String>,
}
