use regex::RegexBuilder;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;

const ASA_STAGE4_SCRIPT_URL: &str = "https://raw.githubusercontent.com/asarubbo/gentoo-stage4/master/autoinstaller-scripts/stage4/stage4";
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let stage4_url = get_stage4_url(Profile::Hardened)?;
    println!("using stage4 url '{}'", stage4_url);

    println!("downloading..");
    download_stage_4(stage4_url)?;
    println!("done downloading..");

    println!("unpacking");
    fs::create_dir_all("chroot_tokyo")?;
    unpack_stage_4("stage4.tar", "chroot_tokyo")?;
    println!("finished unpacking");
    Ok(())
}

#[allow(dead_code)]
enum Profile {
    Standard,
    Hardened,
}

fn get_stage4_url(profile: Profile) -> Result<String> {
    let body = reqwest::blocking::get(ASA_STAGE4_SCRIPT_URL)?.text()?;
    let re = RegexBuilder::new(r#"^#STAGE4_URL="(.*)"\s;.*"$"#)
        .multi_line(true)
        .build()?;
    let captures = re.captures_iter(&body).collect::<Vec<_>>();
    let capture = match profile {
        Profile::Standard => &captures[1],
        Profile::Hardened => &captures[0],
    };
    Ok(capture
        .get(1)
        .ok_or("unable to find url in script")?
        .as_str()
        .to_owned())
}

fn download_stage_4(url: String) -> Result<()> {
    let mut resp = reqwest::blocking::get(url)?;
    let mut fd = File::create("stage4.tar")?;
    resp.copy_to(&mut fd)?;
    Ok(())
}

fn unpack_stage_4(tar_file: &str, target: &str) -> Result<()> {
    // tar xpvf stage3-*.tar.xz --xattrs-include='*.*' --numeric-owner

    let output = Command::new("tar")
        .args(&[
            "xpvf",
            tar_file,
            "-C",
            target,
            "--xattrs-include='*.*'",
            "--numeric-owner",
        ])
        .output()?;

    if !output.status.success() {
        io::stderr().write_all(&output.stderr).unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stage4_url() {
        let url = get_stage4_url(Profile::Standard).expect("failed to get stage4 url");
        assert!(!url.contains("hardened"));
        let url = get_stage4_url(Profile::Hardened).expect("failed to get stage4 url");
        assert!(url.contains("hardened"));
    }
}
