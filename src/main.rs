use regex::RegexBuilder;
use std::error::Error;
use std::path::Path;
use std::process::Command;

const ASA_STAGE4_SCRIPT_URL: &str = "https://raw.githubusercontent.com/asarubbo/gentoo-stage4/master/autoinstaller-scripts/stage4/stage4";
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let stage4_url = get_stage4_url(Profile::Hardened)?;
    println!("using stage4 url '{}'", stage4_url);
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

fn unpack_stage_4<P: AsRef<Path>>(tar_file: &str, target: P) -> Result<()> {
    // tar xpvf stage3-*.tar.xz --xattrs-include='*.*' --numeric-owner

    let output = Command::new("tar")
        .args(&[
            "x",
            "p",
            "v",
            "f",
            tar_file,
            "--xattrs-include='*.*'",
            "--numeric-owner",
        ])
        .output()?;

    output
        .status
        .success()
        .then(|| ())
        .ok_or("failed to unpack tarball")?;
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
