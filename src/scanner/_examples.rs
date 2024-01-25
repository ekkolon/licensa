pub fn example_scan_op() -> anyhow::Result<()> {
  let root = std::env::current_dir()?;
  let candidates = super::scan(root)?;
  if candidates.is_empty() {
    return Ok(());
  }

  let first_candidate = &candidates[2];
  let dotext = &first_candidate
    .ext
    .clone()
    .map(|e| format!(".{}", &e).to_string());

  println!("Extension {}", &dotext.as_ref().unwrap());

  if dotext.is_some() {
    println!("Extension {}", &dotext.as_ref().unwrap());
    let prefix = super::source::SourceHeaders::find_header_prefix_for_extension(
      dotext.as_ref().unwrap(),
    );
    if prefix.is_some() {
      let p = prefix.as_ref().unwrap().to_owned();

      println!(
        "Prefix for extension {}: {:?}",
        &dotext.as_ref().unwrap(),
        &prefix.as_ref().unwrap()
      );

      let file_content =
        std::fs::read_to_string("./src/main.rs").expect("Failed to read file");

      if super::header_checker::contains_copyright_notice(&file_content) {
        println!("File {} has a license header.", "./example.js");
      } else {
        println!("File {} does not contain license header.", "./example.js");
      }
    }
  }

  Ok(())
}
