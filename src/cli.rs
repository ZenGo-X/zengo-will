use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct App {
    #[structopt(short)]
    pub t: u64,

    #[structopt(long)]
    pub persistent_store: PathBuf,

    #[structopt(long)]
    pub cert: PathBuf,
    #[structopt(long)]
    pub key: PathBuf,

    #[structopt(long)]
    pub testator_ca: PathBuf,

    #[structopt(long, default_value = "4949")]
    pub beneficiary_api_port: u16,
    #[structopt(long, default_value = "4950")]
    pub testator_api_port: u16,
}
