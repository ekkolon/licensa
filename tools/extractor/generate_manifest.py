import argparse
import errno
import json
import logging
import os
from pathlib import Path

import requests
from license_header import extract_license_header
from selenium import webdriver
from selenium.webdriver.common.by import By

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

CHOOSEALICENSE_URL = "https://choosealicense.com/appendix/"

GITHUB_LICENSES_URL_BASE = (
    "https://raw.githubusercontent.com/github/choosealicense.com/gh-pages/_licenses"
)


parser = argparse.ArgumentParser(description="Fetch license metadata")
parser.add_argument("-o", "--out-dir", required=True)


def extract_license_text(content: str):
    """
    Extracts license text from the input content.

    Parameters:
        content (str): The input content.

    Returns:
        str: Extracted license text.
    """

    slice_index = content.rfind("---")

    if slice_index != -1:
        # NOTE: Leave whitespaces untouched.
        # Don't use methods like `.lstrip()` or `.rstrip()`.
        result = content[slice_index + 5 :]
        return result
    else:
        return content


def fetch_license_template(template_url: str) -> str:
    res = requests.get(template_url)
    return res.text


def fetch_available_licenses() -> tuple[str, int]:
    """Find and print license file names and SPDX identifiers"""

    # List to store license information in the form of dictionaries
    licenses = []

    # Open the Choose a License GitHub page in the Chrome browser
    driver.get(CHOOSEALICENSE_URL)

    # Find all elements matching the specified CSS selector
    elements = driver.find_elements(
        By.CSS_SELECTOR, 'body > div > table > tbody > tr > th[scope="row"] a'
    )

    # Iterate through each element and extract title and SPDX identifier
    for element in elements:
        title = element.text
        # The last part is the lowercased license SPDX ID
        spdx_id_lower = element.get_attribute("href").split("/").pop()
        template_url = _generate_gh_license_template_url(spdx_id_lower)

        license = {
            "title": title,
            "spdx_id_lower": spdx_id_lower,
            "template_url": template_url,
        }

        licenses.append(license)

    num_licenses = len(licenses)

    # Pretty formatted JSON
    licenses = json.dumps(licenses, indent=2)

    return licenses, num_licenses


def _generate_gh_license_template_url(spdx_id: str):
    return "{}/{}.txt".format(GITHUB_LICENSES_URL_BASE, spdx_id)


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


def _save_template(template: str, path: str):
    lines = template.splitlines(True)
    # Save template file at specified template_path
    # Note: encoding kwarg is required
    with open(path, "wt", encoding="utf-8") as f:
        f.writelines(lines)
        f.close()


def _has_license_header(path: str):
    return os.path.isfile(path)


def _generate_license_manifest(out_dir: str, data):
    out_path = os.path.join(out_dir, LICENSE_MANIFEST)
    with open(out_path, "wt", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    args = parser.parse_args()

    out_dir_base = getattr(args, "out_dir")
    out_dir = os.path.abspath(out_dir_base)
    _check_templates_out_dir(out_dir)

    # Find and print license information
    # =========================================================================
    logging.info("Fetching licenses infos")

    licenses, num_licenses = fetch_available_licenses()
    driver.quit()

    licenses = json.loads(licenses)
    logging.info("Found {} licenses".format(num_licenses))

    # =========================================================================
    logging.info("Start extracting licenses metadata from GitHub repository")

    for license in licenses:
        # Fetch the raw license template from the choosealicense repository
        # and save it locally using the same file name.
        template_url = license["template_url"]
        template = fetch_license_template(template_url)
        meta = extract_license_header(template)

        license["spdx_id"] = meta.spdx_id
        license["title"] = meta.title

        template_path = _build_template_path(meta.spdx_id, out_dir)
        _save_template(template, template_path)
        logging.info("Successfully saved {} license template")

        template_header_path = _build_template_header_path(meta.spdx_id, out_dir)
        license["has_header"] = _has_license_header(template_header_path)

    # =========================================================================
    _generate_license_manifest(out_dir, licenses)
    logging.info("Successfully saved {} file".format(LICENSE_MANIFEST))
