import argparse
import errno
import json
from logger import logger
import os
from pathlib import Path

from selenium import webdriver
from template import TemplateRefStore

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


TEMPLATES_OUT_DIR = "templates"

TEMPLATE_HEADERS_REL_DIR = "headers"

LICENSE_MANIFEST = "licenses.manifest.json"


parser = argparse.ArgumentParser(description="Fetch license metadata")
parser.add_argument("-o", "--out-dir", required=True)


def _is_directory_empty(dir: str) -> bool:
    return len(os.listdir(dir)) == 0


def _check_templates_out_dir(dir: str):
    out_dir = os.path.join(dir, TEMPLATES_OUT_DIR)

    try:
        os.makedirs(out_dir)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise

    if not Path(out_dir).is_dir():
        raise argparse.ArgumentError(None, "'{}' is not a directory".format(out_dir))

    if not _is_directory_empty(out_dir):
        raise argparse.ArgumentError(None, "'{}' is not empty".format(out_dir))


def _build_template_header_path(spdx_id: str, out_dir: str):
    filename = "{}.txt".format(spdx_id.lower())
    return os.path.join(out_dir, TEMPLATE_HEADERS_REL_DIR, filename)


def _build_template_path(spdx_id: str, out_dir: str):
    filename = "{}.txt".format(spdx_id.lower())
    return os.path.join(out_dir, TEMPLATES_OUT_DIR, filename)


def _has_license_header(path: str):
    return os.path.isfile(path)


def _generate_license_manifest(out_dir: str, data):
    out_path = os.path.join(out_dir, LICENSE_MANIFEST)
    with open(out_path, "wt", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    logger.info("Fetching licenses infos")

    args = parser.parse_args()

    out_dir_base = getattr(args, "out_dir")
    out_dir = os.path.abspath(out_dir_base)
    # _check_templates_out_dir(out_dir)

    # Find and print license information
    # =========================================================================
    logger.info("Fetching licenses infos")

    store = TemplateRefStore(driver=driver)
    store.fetch_available_licenses()
    driver.quit()

    licenses = store.licenses
    logger.info("Found {} licenses".format(store.num_refs))

    # =========================================================================
    logger.info("Start extracting licenses metadata from GitHub repository")

    for license in licenses:
        # Fetch the raw license template from the choosealicense repository
        # and save it locally using the same file name.
        license.fetch_template()

        template_path = _build_template_path(license.spdx_id_lower, out_dir)
        # license.save_template(template_path)
        # logging.info("Successfully saved {} license template")

        template_header_path = _build_template_header_path(license.spdx_id, out_dir)
        license.has_header = _has_license_header(template_header_path)

    # =========================================================================
    # _generate_license_manifest(out_dir, licenses)
    logger.info("Successfully saved {} file".format(LICENSE_MANIFEST))
