// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

/// Verify whether a license template exists in local Data directory
/// for the given SPDX identifier.
pub fn confirm_remote_fetch<T>(spdx_id: T) -> io::Result<String>
where
  T: AsRef<str>,
{
  let template = fetch_local_template(spdx_id);
  match template {
    Err(err) => {
      println!("License template does not exist locally, would you like to download it? \n[y/N]");

      let mut input = String::new();
      match io::stdin().lock().lines().next() {
        Some(line) => input = line.unwrap(),
        None => println!("\n"),
      };

      if input.to_lowercase() != "y" {
        println!("Confirmation not received. Exiting.");
        process::exit(1)
      } else {
        Ok(input)
      }
    }
    Ok(template) => Ok(template),
  }
}

pub async fn fetch_remote_template<T>(
  spdx_id: T,
) -> Result<(), Box<dyn std::error::Error>>
where
  T: AsRef<str>,
{
  // Check whether a SPDX license text exists locally
  let license_text = fetch_local_template(&spdx_id);
  if let Ok(license_text) = license_text {
    // License template from local store
    println!(
      "Fetched license template from local store:\n\n{}",
      license_text
    );

    return Ok(());
  };

  // Fetch template from Github repo
  let url = get_remote_template_url(&spdx_id);
  let client = Client::new();
  let response = client.get(url).send().await.unwrap();
  let body = response.text().await.unwrap();
  let license_text = extract_license_text(&body);

  println!(
    "Fetched license template from remote store:\n\n{}",
    license_text
  );

  // Save template to local store
  save_license_text(&spdx_id, &license_text)?;

  Ok(())
}

// FIXME: Don't allocate result
fn extract_license_text(input: &str) -> String {
  if let Some(last_separator) = input.rfind("---") {
    // Remove the newline after the last occurrence of ---
    let result = input[last_separator + 3..].trim_start();
    String::from(result)
  } else {
    // If --- is not found, return the entire input
    String::from(input)
  }
}

fn get_remote_template_url<T>(spdx_id: T) -> String
where
  T: AsRef<str>,
{
  let filename = make_template_filename(&spdx_id);
  format!("{SPDX_LICENSE_DATA_REMOTE_URL}/{}", filename)
}

/// Verify whether a license template exists in local Data directory
/// for the given SPDX identifier.
pub fn fetch_local_template<T>(spdx_id: T) -> io::Result<String>
where
  T: AsRef<str>,
{
  let template_path = get_template_path(spdx_id);
  fs::read_to_string(template_path)
}

pub fn save_license_text<I, T>(spdx_id: I, contents: T) -> io::Result<()>
where
  I: AsRef<str>,
  T: AsRef<str>,
{
  let template_path = get_template_path(&spdx_id);
  // Create templates directory if it doesn't exist
  fs::create_dir_all(templates_dir())?;
  fs::write(template_path.as_path(), contents.as_ref().as_bytes()).unwrap_or_else(
    |err| {
      panic!(
        "\nFailed to write License template for the {} license: {}",
        &spdx_id.as_ref().to_string(),
        err
      )
    },
  );
  println!(
    "\nSuccessfully saved \"{}\" license",
    &template_path.display()
  );

  Ok(())
}

pub fn get_template_path<T>(spdx_id: T) -> PathBuf
where
  T: AsRef<str>,
{
  let filename = make_template_filename(&spdx_id);
  templates_dir().join(filename)
}

fn make_template_filename<T>(spdx_id: T) -> String
where
  T: AsRef<str>,
{
  format!("{}.txt", &spdx_id.as_ref().to_lowercase())
}
