use survivors_reference::Profile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("expected profile path")?;
    let profile = Profile::parse(&std::fs::read_to_string(path)?)?;
    println!(
        "Validated {} classes from {}",
        profile.classes().len(),
        profile.source_sha256()
    );
    Ok(())
}
