//! `black extract-sprites <category> <out>` (A10): ROM sprites -> `<out>/sprites/<category>/`.

pub(crate) mod gfxarc;
pub(crate) mod items;
pub(crate) mod overworld;
pub(crate) mod pokemon;
pub(crate) mod spa;
pub(crate) mod trainers;
pub(crate) mod ui;

use clap::Args as ClapArgs;
use nds_rom::narc::Narc;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Category: pokemon (battle sprites + icons)
    pub category: String,
    pub out: PathBuf,
    /// The black-rom directory (contains files/)
    #[arg(long, default_value = "black-rom")]
    pub rom_root: PathBuf,
    /// Also write a contact sheet PNG of ~20 species (visual check)
    #[arg(long)]
    pub contact: Option<PathBuf>,
    /// Orientation for --contact (front or back)
    #[arg(long, default_value = "front")]
    pub contact_view: String,
}

pub fn run(a: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let files = a.rom_root.join("files");
    match a.category.as_str() {
        "pokemon" => {}
        "trainers" => {
            let r = trainers::extract(&files, &a.out)?;
            println!(
                "trainers: {} sprite sets, {} failures",
                r.sets,
                r.failures.len()
            );
            for (n, e) in &r.failures {
                println!("  {n}: {e}");
            }
            if let Some(p) = &a.contact {
                std::fs::write(p, trainers::contact_sheet(&files)?.to_png()?)?;
            }
            return Ok(());
        }
        "overworld" => {
            let r = overworld::extract(&files, &a.out)?;
            println!(
                "overworld: {} character sets, {} frames, {} failures",
                r.sets,
                r.frames,
                r.failures.len()
            );
            for (n, e) in &r.failures {
                println!("  member {n}: {e}");
            }
            if let Some(p) = &a.contact {
                std::fs::write(p, overworld::contact_sheet(&files)?.to_png()?)?;
            }
            return Ok(());
        }
        "items" | "icons" | "battle-ui" => {
            let (n, fails) = if a.category == "items" {
                let r = items::extract(&files, &a.out)?;
                (r.items, r.failures)
            } else {
                let r = ui::extract(&files, &a.out, &a.category)?;
                (r.images, r.failures)
            };
            println!("{}: {n} images, {} failures", a.category, fails.len());
            for (k, e) in &fails {
                println!("  {k}: {e}");
            }
            if let Some(p) = &a.contact {
                std::fs::write(p, ui::contact_sheet(&files)?.to_png()?)?;
            }
            return Ok(());
        }
        "battle-hud" | "menus" => {
            let r = gfxarc::extract(&files, &a.out, &a.category)?;
            println!(
                "{}: {} images in {} atlases, {} failures",
                a.category,
                r.images,
                r.files,
                r.failures.len()
            );
            for (k, e) in &r.failures {
                println!("  {k}: {e}");
            }
            if let Some(p) = &a.contact {
                std::fs::write(p, gfxarc::contact_sheet(&files, &a.category)?.to_png()?)?;
            }
            return Ok(());
        }
        "effects" => {
            let r = spa::extract(&files, &a.out)?;
            println!(
                "effects: {} SPA files, {} textures, {} failures",
                r.spa_files,
                r.textures,
                r.failures.len()
            );
            for (k, e) in &r.failures {
                println!("  {k}: {e}");
            }
            if let Some(p) = &a.contact {
                std::fs::write(p, spa::contact_sheet(&files)?.to_png()?)?;
            }
            return Ok(());
        }
        c => {
            return Err(format!(
                "unknown sprite category '{c}' (known: pokemon, trainers, overworld, items, icons, battle-ui, battle-hud, menus, effects)"
            )
            .into())
        }
    }
    let rep = pokemon::extract(&files, &a.out)?;
    println!(
        "pokemon: {} blocks, {} sprite sets, {} icons ({} frames) -> {}",
        rep.blocks,
        rep.sprites,
        rep.icons,
        rep.icon_frames,
        a.out.join("sprites/pokemon").display()
    );
    for (b, e) in &rep.failures {
        println!("  block {b}: {e}");
    }
    if let Some(p) = &a.contact {
        let narc = Narc::parse(&std::fs::read(files.join("a/0/0/4"))?)?;
        let blocks = [
            1, 4, 7, 25, 54, 94, 129, 150, 197, 249, 282, 330, 376, 445, 448, 521, 571, 635, 646,
            649,
        ];
        let o = if a.contact_view == "back" {
            pokemon::Orientation::Back
        } else {
            pokemon::Orientation::Front
        };
        let img = pokemon::contact_sheet(&narc, &blocks, o)?;
        std::fs::write(p, img.to_png()?)?;
        println!("contact sheet -> {}", p.display());
    }
    Ok(())
}
