use std::path::Path;
use std::process::exit;

use clap::{App,Arg,SubCommand,ArgMatches};
use clap::AppSettings::*;
use libcitadel::{Result, ResourceImage, Logger, LogLevel, Partition, KeyPair, ImageHeader, util};
use hex;

pub fn main(args: Vec<String>) {

    let app = App::new("citadel-image")
        .about("Citadel update image builder")
        .settings(&[ArgRequiredElseHelp,ColoredHelp, DisableHelpSubcommand, DisableVersion, DeriveDisplayOrder])

        .subcommand(SubCommand::with_name("metainfo")
            .about("Display metainfo variables for an image file")
            .arg(Arg::with_name("path")
                .required(true)
                .help("Path to image file")))

        .subcommand(SubCommand::with_name("info")
            .about("Display metainfo variables for an image file")
            .arg(Arg::with_name("path")
                .required(true)
                .help("Path to image file")))

        .subcommand(SubCommand::with_name("generate-verity")
            .about("Generate dm-verity hash tree for an image file")
            .arg(Arg::with_name("path")
                .required(true)
                .help("Path to image file")))

        .subcommand(SubCommand::with_name("verify")
            .about("Verify dm-verity hash tree for an image file")
            .arg(Arg::with_name("path")
                .required(true)
                .help("Path to image file")))

        .subcommand(SubCommand::with_name("install-rootfs")
            .about("Install rootfs image file to a partition")
            .arg(Arg::with_name("choose")
                .long("just-choose")
                .help("Don't install anything, just show which partition would be chosen"))
            .arg(Arg::with_name("skip-sha")
                .long("skip-sha")
                .help("Skip verification of header sha256 value"))
            .arg(Arg::with_name("no-prefer")
                .long("no-prefer")
                .help("Don't set PREFER_BOOT flag"))
            .arg(Arg::with_name("path")
                .required_unless("choose")
                .help("Path to image file")))

        .subcommand(SubCommand::with_name("genkeys")
            .about("Generate a pair of keys"))

        .subcommand(SubCommand::with_name("decompress")
            .about("Decompress a compressed image file")
            .arg(Arg::with_name("path")
                .required(true)
                .help("Path to image file")))

        .subcommand(SubCommand::with_name("bless")
            .about("Mark currently mounted rootfs partition as successfully booted"))

        .subcommand(SubCommand::with_name("verify-shasum")
            .about("Verify the sha256 sum of the image")
            .arg(Arg::with_name("path")
                .required(true)
                .help("Path to image file")));

    Logger::set_log_level(LogLevel::Debug);

    let matches = app.get_matches_from(args);
    let result = match matches.subcommand() {
        ("metainfo", Some(m)) => metainfo(m),
        ("info", Some(m)) => info(m),
        ("generate-verity", Some(m)) => generate_verity(m),
        ("verify", Some(m)) => verify(m),
        ("sign-image", Some(m)) => sign_image(m),
        ("genkeys", Some(_)) => genkeys(),
        ("decompress", Some(m)) => decompress(m),
        ("verify-shasum", Some(m)) => verify_shasum(m),
        ("install-rootfs", Some(m)) => install_rootfs(m),
        ("install", Some(m)) => install_image(m),
        ("bless", Some(_)) => bless(),
        _ => Ok(()),
    };

    if let Err(ref e) = result {
        println!("Error: {}", e);
        exit(1);
    }
}

fn info(arg_matches: &ArgMatches) -> Result<()> {
    let img = load_image(arg_matches)?;
    print!("{}",String::from_utf8(img.header().metainfo_bytes())?);
    info_signature(&img)?;
    Ok(())
}

fn info_signature(img: &ResourceImage) -> Result<()> {
    if img.header().has_signature() {
        println!("Signature: {}", hex::encode(&img.header().signature()));
    } else {
        println!("Signature: No Signature");
    }
    match img.header().public_key()? {
        Some(pubkey) => {
            if img.header().verify_signature(pubkey) {
                println!("Signature is valid");
            } else {
                println!("Signature verify FAILED");
            }
        },
        None => { println!("No public key found for channel '{}'", img.metainfo().channel()) },
    }
   Ok(())
}
fn metainfo(arg_matches: &ArgMatches) -> Result<()> {
    let img = load_image(arg_matches)?;
    print!("{}",String::from_utf8(img.header().metainfo_bytes())?);
    Ok(())
}

fn generate_verity(arg_matches: &ArgMatches) -> Result<()> {
    let img = load_image(arg_matches)?;
    if img.has_verity_hashtree() {
        info!("Image already has dm-verity hashtree appended, doing nothing.");
    } else {
        img.generate_verity_hashtree()?;
    }
    Ok(())
}

fn verify(arg_matches: &ArgMatches) -> Result<()> {
    let img = load_image(arg_matches)?;
    let ok = img.verify_verity()?;
    if ok {
        info!("Image verification succeeded");
    } else {
        warn!("Image verification FAILED!");
    }
    Ok(())
}

fn verify_shasum(arg_matches: &ArgMatches) -> Result<()> {
    let img = load_image(arg_matches)?;
    let shasum = img.generate_shasum()?;
    if shasum == img.metainfo().shasum() {
        info!("Image has correct sha256sum: {}", shasum);
    } else {
        info!("Image sha256 sum does not match metainfo:");
        info!("     image: {}", shasum);
        info!("  metainfo: {}", img.metainfo().shasum())
    }
    Ok(())
}

fn load_image(arg_matches: &ArgMatches) -> Result<ResourceImage> {
    let path = arg_matches.value_of("path").expect("path argument missing");
    if !Path::new(path).exists() {
        bail!("Cannot load image {}: File does not exist", path);
    }
    let img = ResourceImage::from_path(path)?;
    if !img.is_valid_image() {
        bail!("File {} is not a valid image file", path);
    }
    Ok(img)
}

fn install_rootfs(arg_matches: &ArgMatches) -> Result<()> {
    if arg_matches.is_present("choose") {
        let _ = choose_install_partition(true)?;
        return Ok(())
    }

    let img = load_image(arg_matches)?;

    if !arg_matches.is_present("skip-sha") {
        info!("Verifying sha256 hash of image");
        let shasum = img.generate_shasum()?;
        if shasum != img.metainfo().shasum() {
            bail!("image file does not have expected sha256 value");
        }
    }

    let partition = choose_install_partition(true)?;

    if !arg_matches.is_present("no-prefer") {
        clear_prefer_boot()?;
        img.header().set_flag(ImageHeader::FLAG_PREFER_BOOT);
    }
    img.write_to_partition(&partition)?;
    Ok(())
}

fn clear_prefer_boot() -> Result<()> {
    for mut p in Partition::rootfs_partitions()? {
        if p.is_initialized() && p.header().has_flag(ImageHeader::FLAG_PREFER_BOOT) {
            p.clear_flag_and_write(ImageHeader::FLAG_PREFER_BOOT)?;
        }
    }
    Ok(())
}

fn sign_image(arg_matches: &ArgMatches) -> Result<()> {
    let _img = load_image(arg_matches)?;
    info!("Not implemented yet");
    Ok(())
}

fn install_image(arg_matches: &ArgMatches) -> Result<()> {
    let source = arg_matches.value_of("path").expect("path argument missing");
    let img = load_image(arg_matches)?;
    let _hdr = img.header();
    let metainfo = img.metainfo();

    // XXX verify signature?

    if !(metainfo.image_type() == "kernel" || metainfo.image_type() == "extra") {
        bail!("Cannot install image type {}", metainfo.image_type());
    }

    let shasum = img.generate_shasum()?;
    if shasum != img.metainfo().shasum() {
        bail!("Image shasum does not match metainfo");
    }

    img.generate_verity_hashtree()?;

    let filename = if metainfo.image_type() == "kernel" {
        let kernel_version = match metainfo.kernel_version() {
            Some(version) => version,
            None => bail!("Kernel image does not have a kernel version field in metainfo"),
        };
        if kernel_version.chars().any(|c| c == '/') {
            bail!("Kernel version field has / char");
        }
        format!("citadel-kernel-{}-{:03}.img", kernel_version, metainfo.version())
    } else {
        format!("citadel-extra-{:03}.img", metainfo.version())
    };

    if !metainfo.channel().chars().all(|c| c.is_ascii_lowercase()) {
        bail!("Refusing to build path from strange channel name {}", metainfo.channel());
    }
    let image_dir = Path::new("/storage/resources").join(metainfo.channel());
    let image_dest = image_dir.join(filename);
    if image_dest.exists() {
        rotate(&image_dest)?;
    }
    util::rename(source, &image_dest)
}

fn rotate(path: &Path) -> Result<()> {
    if !path.exists() || path.file_name().is_none() {
        return Ok(());
    }
    let filename = path.file_name().unwrap();
    let dot_zero = path.with_file_name(format!("{}.0", filename.to_string_lossy()));
    util::remove_file(&dot_zero)?;
    util::rename(path, &dot_zero)
}

fn genkeys() -> Result<()> {
    let keypair = KeyPair::generate();
    println!("keypair = \"{}\"", keypair.to_hex());
    Ok(())
}

fn decompress(arg_matches: &ArgMatches) -> Result<()> {
    let img = load_image(arg_matches)?;
    if !img.is_compressed() {
        info!("Image is not compressed, not decompressing.");
    } else {
        img.decompress(false)?;
    }
    Ok(())
}

fn bless() -> Result<()> {
    for mut p in Partition::rootfs_partitions()? {
        if p.is_initialized() && p.is_mounted() {
            p.bless()?;
            return Ok(());
        }
    }
    warn!("No mounted partition found to bless");
    Ok(())
}

fn bool_to_yesno(val: bool) -> &'static str {
    if val {
        "YES"
    } else {
        " NO"
    }
}

fn choose_install_partition(verbose: bool) -> Result<Partition> {
    let partitions = Partition::rootfs_partitions()?;

    if verbose {
        for p in &partitions {
            info!("Partition: {}  (Mounted: {}) (Empty: {})",
                  p.path().display(),
                  bool_to_yesno(p.is_mounted()),
                  bool_to_yesno(!p.is_initialized()));
        }
    }

    for p in &partitions {
        if !p.is_mounted() && !p.is_initialized() {
            if verbose {
                info!("Choosing {} because it is empty and not mounted", p.path().display());
            }
            return Ok(p.clone())
        }
    }
    for p in &partitions {
        if !p.is_mounted() {
            if verbose {
                info!("Choosing {} because it is not mounted", p.path().display());
                info!("Header metainfo:");
                print!("{}",String::from_utf8(p.header().metainfo_bytes())?);
            }
            return Ok(p.clone())
        }
    }
    bail!("No suitable install partition found")
}
