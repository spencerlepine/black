//! `black`: build, extract and verify the Pokemon Black ROM.

use clap::{Args, Parser, Subcommand};
use nds_rom::build::{self, BuildInputs};
use nds_rom::error::{read_file, write_file};
use nds_rom::{banner, blz, compare, module, overlay, pack, sha1_hex, Rom};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "black", version, about = "Pokemon Black (NDS) ROM tooling")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print the header, file system and overlay summary of a ROM
    Info { rom: PathBuf },
    /// Extract header, binaries, overlays, banner, FNT and data files from a ROM
    Extract { rom: PathBuf, outdir: PathBuf },
    /// Assemble a ROM from built artifacts (make_rom.py)
    Build(Box<BuildArgs>),
    /// Compress ARM9 and overlays into their ROM storage form (compress_modules.py)
    Pack(PackArgs),
    /// Check files against a sha1sum-style list
    Verify {
        /// ROM to check against the first hash in the list; omit to check every listed file
        rom: Option<PathBuf>,
        #[arg(long)]
        sha1_file: PathBuf,
        /// Directory the paths in the list are relative to
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// BLZ compression tools
    Blz {
        #[command(subcommand)]
        op: BlzCmd,
    },
    /// Print an overlay table (parse_overlay_table.py)
    Table {
        table: PathBuf,
        #[arg(long, conflicts_with = "csv")]
        lsf: bool,
        #[arg(long)]
        csv: bool,
    },
    /// List the autoload blocks of a decompressed ARM9/ARM7 module
    Modules {
        image: PathBuf,
        /// RAM load address of the image
        #[arg(long, default_value = "0x02004000", value_parser = parse_u32)]
        load: u32,
    },
    /// Convert banner icons to PNG and back (banner.py)
    Banner {
        #[command(subcommand)]
        op: BannerCmd,
    },
    /// Compare built artifacts with original ones
    Compare {
        #[command(subcommand)]
        op: CompareCmd,
    },
}

#[derive(Args)]
struct BuildArgs {
    /// Project root; defaults for the other paths are taken relative to it
    #[arg(long, default_value = ".")]
    project: PathBuf,
    #[arg(long, default_value = "build/black.us")]
    build: PathBuf,
    #[arg(long)]
    header: Option<PathBuf>,
    #[arg(long)]
    fnt: Option<PathBuf>,
    #[arg(long)]
    twl_tail: Option<PathBuf>,
    /// Directory of overlays in their ROM storage form (default <build>/packed)
    #[arg(long)]
    overlays: Option<PathBuf>,
    /// Overlay file name pattern; `{n}` is the index, `{n:04}` zero padded
    #[arg(long, default_value = "OVY_{n}.sbin")]
    overlay_pattern: String,
    /// The compressed ARM9 (default <build>/packed/arm9.bin)
    #[arg(long)]
    arm9: Option<PathBuf>,
    #[arg(long)]
    arm7: Option<PathBuf>,
    /// Overlay table (default <build>/packed/main_table.sbin)
    #[arg(long)]
    table: Option<PathBuf>,
    #[arg(long)]
    banner: Option<PathBuf>,
    #[arg(long)]
    files: Option<PathBuf>,
    #[arg(long)]
    data_dir: Option<PathBuf>,
    /// Print the base ROM's file order (from --header and --fnt) and exit
    #[arg(long)]
    list_order: bool,
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct PackArgs {
    #[arg(long, default_value = "build/black.us")]
    build: PathBuf,
    /// Ids the ROM stores uncompressed, one per line
    #[arg(long, default_value = "black.us/overlay_compression.txt")]
    policy: PathBuf,
    #[arg(long)]
    outdir: PathBuf,
    /// ARM9 RAM address (default: first word of <build>/main_defs.sbin)
    #[arg(long, value_parser = parse_u32)]
    load: Option<u32>,
}

#[derive(Subcommand)]
enum BlzCmd {
    /// Compress a file; exits with an error if it cannot be shrunk
    Compress {
        input: PathBuf,
        output: PathBuf,
        /// Leading bytes to leave uncompressed (0x4000 for the ARM9)
        #[arg(long, default_value = "0", value_parser = parse_u32)]
        floor: u32,
        /// Shorthand for --floor 0x4000
        #[arg(short = '9', long)]
        arm9: bool,
    },
    /// Decompress a file (blz.py)
    Decompress { input: PathBuf, output: PathBuf },
}

#[derive(Subcommand)]
enum BannerCmd {
    Extract {
        banner: PathBuf,
        #[arg(long)]
        dir: PathBuf,
    },
    Build {
        #[arg(long)]
        dir: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum CompareCmd {
    /// Built (decompressed) ARM9 against the original
    Arm9 { built: PathBuf, original: PathBuf },
    /// Built OVY_<n>.sbin files against extracted overlays
    Overlays {
        #[arg(long)]
        table: PathBuf,
        #[arg(long)]
        overlays: PathBuf,
        #[arg(long)]
        built: PathBuf,
        #[arg(long, default_value_t = 20)]
        show: usize,
    },
    /// Built overlay table against the original
    Table {
        built: PathBuf,
        original: PathBuf,
        #[arg(long, default_value_t = 6)]
        show: usize,
    },
}

fn parse_u32(s: &str) -> std::result::Result<u32, String> {
    let s = s.trim();
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16),
        None => s.parse(),
    }
    .map_err(|e| e.to_string())
}

fn info(path: &Path) -> Result<()> {
    let rom = Rom::open(path)?;
    let h = &rom.header;
    println!("title:        {}", h.title_str());
    println!(
        "game code:    {}  maker {}  version {}",
        h.game_code_str(),
        String::from_utf8_lossy(&h.maker_code),
        h.rom_version
    );
    println!(
        "unit code:    {:#04x}{}",
        h.unit_code,
        if h.is_dsi() { " (DSi enhanced)" } else { "" }
    );
    println!(
        "size:         {:#x} bytes (header: NTR {:#x}, total {:#x})",
        rom.data.len(),
        h.ntr_rom_size,
        h.twl_rom_size
    );
    println!(
        "header crc:   {:#06x} ({})  logo crc {:#06x} ({})",
        h.header_crc,
        if h.header_crc_valid() { "ok" } else { "BAD" },
        h.logo_crc,
        if h.logo_crc_valid() { "ok" } else { "BAD" }
    );
    for (name, b) in [("ARM9", h.arm9), ("ARM7", h.arm7)] {
        println!(
            "{name}:         offset {:#08x}  entry {:#010x}  ram {:#010x}  size {:#x}",
            b.rom_offset, b.entry, b.ram_address, b.size
        );
    }
    println!(
        "FNT:          offset {:#08x}  size {:#x}",
        h.fnt.offset, h.fnt.size
    );
    println!(
        "FAT:          offset {:#08x}  size {:#x}  ({} files)",
        h.fat.offset,
        h.fat.size,
        h.file_count()
    );
    println!(
        "ARM9 ovt:     offset {:#08x}  {} overlays",
        h.arm9_overlays.offset,
        h.arm9_overlay_count()
    );
    println!(
        "banner:       offset {:#08x}  {:#x} bytes",
        h.banner_offset,
        rom.banner()?.len()
    );
    println!("data files:   {}", rom.data_files().len());
    println!(
        "DSi tail:     offset {:#x}  {:#x} bytes",
        rom.twl_tail_offset(),
        rom.data.len().saturating_sub(rom.twl_tail_offset())
    );
    if let Ok(overlays) = rom.arm9_overlays() {
        let compressed = overlays.iter().filter(|o| o.compressed()).count();
        println!(
            "overlays:     {} ({} compressed)",
            overlays.len(),
            compressed
        );
    }
    if let Ok(arm9) = rom.arm9() {
        // the ARM9 params only make sense once the static module is decompressed
        if let Ok(p) = module::find_module_params(arm9) {
            println!(
                "ARM9 params:  at {:#x}, compressed end {:#010x}",
                p.offset, p.compressed_static_end
            );
        } else if let Ok(d) = blz::decompress(arm9) {
            if let Ok(p) = module::find_module_params(&d) {
                println!(
                    "ARM9 params:  at {:#x} (BLZ compressed, {:#x} bytes unpacked)",
                    p.offset,
                    d.len()
                );
            }
        }
    }
    println!("sha1:         {}", sha1_hex(&rom.data));
    Ok(())
}

fn build_cmd(a: BuildArgs) -> Result<()> {
    let mut d = build::default_inputs(&a.project, &a.project.join(&a.build));
    if let Some(overlays) = a.overlays {
        d.overlays = overlays;
    }
    let inputs = BuildInputs {
        header: a.header.unwrap_or(d.header),
        fnt: a.fnt.unwrap_or(d.fnt),
        twl_tail: a.twl_tail.unwrap_or(d.twl_tail),
        arm9: a.arm9.unwrap_or(d.arm9),
        arm7: a.arm7.unwrap_or(d.arm7),
        table: a.table.unwrap_or(d.table),
        banner: a.banner.unwrap_or(d.banner),
        overlays: d.overlays,
        overlay_pattern: a.overlay_pattern,
        files: a.files.unwrap_or(d.files),
        data_dir: a.data_dir.unwrap_or(d.data_dir),
    };
    if a.list_order {
        for name in build::list_order(&read_file(&inputs.header)?, &read_file(&inputs.fnt)?)? {
            println!("{name}");
        }
        return Ok(());
    }
    let output = a.output.ok_or("building needs --output")?;
    let (rom, report) = build::assemble(&inputs)?;
    write_file(&output, &rom)?;
    println!(
        "wrote {}: {} bytes, {} files",
        output.display(),
        report.size,
        report.files
    );
    Ok(())
}

fn pack_cmd(a: PackArgs) -> Result<()> {
    let policy = pack::parse_policy(&String::from_utf8_lossy(&read_file(&a.policy)?))?;
    let load = match a.load {
        Some(l) => l,
        None => {
            let defs = read_file(a.build.join("main_defs.sbin"))?;
            u32::from_le_bytes(
                defs.get(..4)
                    .ok_or("main_defs.sbin too short")?
                    .try_into()?,
            )
        }
    };
    let packed = pack::pack_build_dir(&a.build, &policy, load)?;
    pack::write_packed(&packed, &a.outdir)?;
    println!(
        "compressed {} overlays, {} left raw, arm9 {} bytes",
        packed.compressed,
        packed.raw,
        packed.arm9.len()
    );
    Ok(())
}

fn verify(rom: Option<PathBuf>, list: &Path, root: &Path) -> Result<bool> {
    let text = String::from_utf8_lossy(&read_file(list)?).to_string();
    let entries: Vec<(&str, &str)> = text
        .lines()
        .filter_map(|l| {
            let (hash, path) = l.split_once(' ')?;
            Some((hash, path.trim_start_matches([' ', '*'])))
        })
        .collect();
    let mut ok = true;
    if let Some(rom) = rom {
        let (want, _) = *entries.first().ok_or("empty sha1 list")?;
        let got = sha1_hex(&read_file(&rom)?);
        ok = got == want;
        println!("{}: {}", rom.display(), if ok { "OK" } else { "FAILED" });
        if !ok {
            println!("  expected {want}\n  actual   {got}");
        }
    } else {
        for (want, path) in entries {
            let good = read_file(root.join(path))
                .map(|d| sha1_hex(&d) == want)
                .unwrap_or(false);
            ok &= good;
            if !good {
                println!("{path}: FAILED");
            }
        }
        if ok {
            println!("all files match {}", list.display());
        }
    }
    Ok(ok)
}

fn run(cli: Cli) -> Result<bool> {
    match cli.command {
        Cmd::Info { rom } => info(&rom)?,
        Cmd::Extract { rom, outdir } => {
            let rom = Rom::open(rom)?;
            let r = nds_rom::rom::extract(&rom, &outdir)?;
            println!(
                "extracted {} overlays, {} data files and {:#x} bytes of DSi tail to {}",
                r.overlays,
                r.data_files,
                r.tail_bytes,
                outdir.display()
            );
        }
        Cmd::Build(a) => build_cmd(*a)?,
        Cmd::Pack(a) => pack_cmd(a)?,
        Cmd::Verify {
            rom,
            sha1_file,
            root,
        } => return verify(rom, &sha1_file, &root),
        Cmd::Blz { op } => match op {
            BlzCmd::Compress {
                input,
                output,
                floor,
                arm9,
            } => {
                let floor = if arm9 { 0x4000 } else { floor as usize };
                let data = read_file(&input)?;
                let packed = blz::compress(&data, floor).ok_or("data does not shrink under BLZ")?;
                write_file(&output, &packed)?;
                println!("{} -> {} bytes", data.len(), packed.len());
            }
            BlzCmd::Decompress { input, output } => {
                let data = blz::decompress(&read_file(&input)?)?;
                write_file(&output, &data)?;
                println!("{} bytes", data.len());
            }
        },
        Cmd::Table { table, lsf, csv } => {
            let overlays = overlay::parse_table(&read_file(table)?)?;
            if lsf {
                print!("{}", overlay::format_lsf(&overlays));
            } else if csv {
                print!("{}", overlay::format_csv(&overlays));
            } else {
                print!("{}", overlay::format_listing(&overlays));
            }
        }
        Cmd::Modules { image, load } => {
            let data = read_file(image)?;
            let p = module::find_module_params(&data)?;
            println!("module params at {:#x}: {p:#x?}", p.offset);
            let (start, blocks) = module::autoload_blocks(&data, load)?;
            println!("autoload payload starts at {start:#010x}");
            for (i, b) in blocks.iter().enumerate() {
                println!(
                    "  {i}: load {:#010x} size {:#x} bss {:#x} sinit {:#010x}",
                    b.load, b.size, b.bss, b.sinit
                );
            }
        }
        Cmd::Banner { op } => match op {
            BannerCmd::Extract { banner: path, dir } => {
                let n = banner::extract(&read_file(path)?, &dir)?;
                println!("wrote {n} PNGs and banner.meta to {}", dir.display());
            }
            BannerCmd::Build { dir, output } => {
                write_file(&output, &banner::build(&dir)?)?;
                println!("wrote {}", output.display());
            }
        },
        Cmd::Compare { op } => return compare_cmd(op),
    }
    Ok(true)
}

fn compare_cmd(op: CompareCmd) -> Result<bool> {
    match op {
        CompareCmd::Arm9 { built, original } => {
            let r = compare::compare_arm9(&read_file(built)?, &read_file(original)?);
            println!("{r}");
            Ok(r.matches())
        }
        CompareCmd::Overlays {
            table,
            overlays,
            built,
            show,
        } => {
            let table = read_file(table)?;
            let r = compare::compare_overlays(
                &table,
                show,
                |i| read_file(overlays.join(format!("overlay_{i:04}.bin"))).ok(),
                |i| read_file(built.join(format!("OVY_{i}.sbin"))).ok(),
            )?;
            println!("{r}");
            Ok(r.matches())
        }
        CompareCmd::Table {
            built,
            original,
            show,
        } => {
            let r = compare::compare_table(&read_file(built)?, &read_file(original)?, show);
            println!("{r}");
            Ok(r.matches())
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
