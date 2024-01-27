use crossbeam_channel::Receiver;

use super::scan::{FileEntry, Scan, ScanConfig};

pub fn example_scan_op() -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let scan_config = ScanConfig {
        limit: 100,
        exclude: None,
        root,
    };
    let scan = Scan::new(scan_config);
    let result = scan.run();

    let candidates = scan.run()?;
    if candidates.is_empty() {
        return Ok(());
    }

    let first_candidate = &candidates[2];
    let dotext = &first_candidate
        .extension
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

            if super::header_checker::contains_copyright_notice(file_content) {
                println!("File ./example.js has a license header.");
            } else {
                println!("File ./example.js does not contain license header.");
            }
        }
    }

    Ok(())
}

pub fn example_scan_parallel() -> anyhow::Result<Receiver<FileEntry>> {
    let root = std::env::current_dir()?;

    let scan_config = ScanConfig {
        limit: 100,
        exclude: None,
        root,
    };

    let scan = Scan::new(scan_config);
    let result = scan.run_parallel()?;

    Ok(result)
}
