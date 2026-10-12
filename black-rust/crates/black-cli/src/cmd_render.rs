//! `black sdat render <sdat> <seq-id> <out.wav> [--loops N]` (A9).

use clap::Args;
use nds_rom::error::{read_file, write_file};
use nds_rom::sdat::render::{RenderOptions, Renderer};
use nds_rom::sdat::Sdat;
use std::path::PathBuf;

#[derive(Args)]
pub struct RenderArgs {
    sdat: PathBuf,
    /// SEQ id (1000-1178 BGM, 1300-1331 ME, 1350-2075 SE)
    seq: usize,
    out: PathBuf,
    /// Passes through the loop body (after the intro)
    #[arg(long, default_value_t = 2)]
    loops: u32,
    /// Cap on song length in seconds
    #[arg(long, default_value_t = 600.0)]
    max_seconds: f64,
}

pub fn run(a: &RenderArgs) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = read_file(&a.sdat)?;
    let sdat = Sdat::parse(&bytes)?;
    let opts = RenderOptions {
        loops: a.loops,
        max_seconds: a.max_seconds,
        ..RenderOptions::default()
    };
    let r = Renderer::new(&sdat).render(a.seq, &opts)?;
    write_file(&a.out, &r.to_wav())?;
    println!(
        "SEQ {}: {} frames at {} Hz ({:.1}s), loop {:?} -> {}",
        a.seq,
        r.pcm.frames(),
        r.pcm.sample_rate,
        r.pcm.frames() as f64 / f64::from(r.pcm.sample_rate),
        r.pcm.looping,
        a.out.display()
    );
    Ok(())
}
