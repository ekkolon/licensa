import argparse
import errno
import logging
import os
import sys
from pathlib import Path

from rich.logging import RichHandler
from selenium import webdriver
from template import LicenseStore

logging.basicConfig(
    level=logging.INFO, format="%(message)s", datefmt="[%X]", handlers=[RichHandler()]
)

log = logging.getLogger("rich")


# Create a new instance of the Chrome driver with specific options
options = webdriver.ChromeOptions()
options.add_experimental_option("excludeSwitches", ["enable-logging"])
options.add_argument("--log-level=3")
options.add_argument("--ignore-certificate-errors")
options.add_argument("--disable-extensions")
options.add_argument("--disable-gpu")
options.add_argument("--no-sandbox")  # linux only
options.add_argument("--headless=new")

driver = webdriver.Chrome(options=options)

DEFAULT_TEMPLATES_DIR = "templates"

DEFAULT_HEADERS_DIR = "headers"

DEFAULT_METADATA_FILENAME = "metadata.json"


def _raise_and_exit(msg: str):
    exception = SystemExit(msg)
    log.error(str(exception))
    sys.exit(1)


def _validate_out_dir(dir: str):
    out_dir = os.path.join(dir, DEFAULT_TEMPLATES_DIR)

    try:
        os.makedirs(out_dir)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

    if not Path(out_dir).is_dir():
        _raise_and_exit("'{}' is not a directory".format(out_dir))

    if not len(os.listdir(out_dir)) == 0:
        _raise_and_exit("Directory '{}' must be empty".format(out_dir))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fetch license metadata")
    parser.add_argument("-o", "--out-dir", required=True)

    args = parser.parse_args()

    out_dir_base = getattr(args, "out_dir")
    out_dir = os.path.abspath(out_dir_base)
    _validate_out_dir(out_dir)

    # Find and print license information
    # =========================================================================
    license_store = LicenseStore(
        driver=driver,
        out_dir=out_dir,
        manifest_name=DEFAULT_METADATA_FILENAME,
        templates_dir=DEFAULT_TEMPLATES_DIR,
        headers_dir=DEFAULT_HEADERS_DIR,
    )

    log.info("Fetching license metadata...")
    license_store.fetch_metadata()
    num_licenses = license_store.num_licenses
    log.info("Found metadata for {} licenses".format(num_licenses))

    driver.quit()

    # =========================================================================
    log.info("Fetching license templates...")
    license_store.fetch_templates()
    log.info("Succesfully fetched templates for {} licenses".format(num_licenses))

    # =========================================================================
    log.info("Saving license templates...")
    license_store.save_templates()
    log.info('License templates saved at "{}"'.format(license_store.out_dir))

    # =========================================================================
    license_store.generate_manifest()
    log.info('Manifest file saved at: "{}"'.format(license_store.manifest_path))
