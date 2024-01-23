import json
import os

import requests
from selenium.webdriver.chrome.webdriver import WebDriver

# from selenium.webdriver.chrome.webdriver import WebDriver
from selenium.webdriver.common.by import By

GITHUB_LICENSES_URL_BASE = (
    "https://raw.githubusercontent.com/github/choosealicense.com/gh-pages/_licenses"
)

CHOOSEALICENSE_URL = "https://choosealicense.com/appendix/"


class LicenseRef:
    def __init__(self, name: str, spdx_id_lower: str):
        self.name = name
        self.spdx_id_lower = spdx_id_lower
        self.template_url = self._generate_template_url()
        self.has_header = False
        self.spdx_id = None
        self.template: str = None
        self.template_path = None
        self.header_path = None
        self.nickname = None

    def fetch_template(self):
        res = requests.get(self.template_url)
        self.template = res.text
        self._extract_meta()
        return self.template

    @property
    def data(self):
        return {
            "has_header": self.has_header,
            "spdx_id": self.spdx_id,
            "name": self.name,
            "template_url": self.template_url,
            "nickname": self.nickname
            # TODO: Add template interpolate fields
        }

    def save_template(self):
        if self.template is None:
            raise ValueError("Cannot save empty license template")

        if self.template_path is None:
            raise ValueError("Template path must be valid")

        lines = self.template.splitlines(True)
        # Save template file at specified template_path
        # Note: encoding kwarg is required
        with open(self.template_path, "wt", encoding="utf-8") as f:
            f.writelines(lines)
            f.close()

    def _extract_meta(self):
        """
        Extracts license metadata from a multiline string.

        Parameters:
            content (str): Multiline string containing license metadata.

        Returns:
            LicenseHeader: Extracted license metadata.
        """

        if self.template is None:
            raise ValueError("Cannot access template before it has been fetched")

        lines = self.template.splitlines()

        # Extract the title
        title_line = next((line for line in lines if line.startswith("title:")), None)
        self.name = (
            title_line.split(": ", 1)[1].strip()
            if title_line
            else self._invalid_field_error("title")
        )

        # Extract the SPDX-Identifier
        spdx_id_line = next(
            (line for line in lines if line.startswith("spdx-id:")), None
        )
        self.spdx_id = (
            spdx_id_line.split(": ", 1)[1].strip()
            if spdx_id_line
            else self._invalid_field_error("spdx_id")
        )

        # Extract the license's nickname, if available
        nickname_line = next(
            (line for line in lines if line.startswith("nickname:")), None
        )
        self.nickname = (
            nickname_line.split(": ", 1)[1].strip() if nickname_line else None
        )

    def _invalid_field_error(field: str) -> str:
        """
        Generates an error message for an invalid field.

        Parameters:
            field (str): The invalid field name.

        Returns:
            str: Error message.
        """
        return f"Failed to determine license header field: {field}"

    def _generate_template_url(self):
        return "{}/{}.txt".format(GITHUB_LICENSES_URL_BASE, self.spdx_id_lower)


class LicenseStore:
    def __init__(
        self,
        driver: WebDriver,
        out_dir: str,
        manifest_name,
        templates_dir,
        headers_dir,
    ):
        self.driver = driver
        self.out_dir = out_dir
        self.manifest_name = manifest_name
        self.templates_dir = templates_dir
        self.headers_dir = headers_dir
        self.licenses: list[LicenseRef] = []
        self.num_licenses = 0

        # Store SPDX ids for quick lookup
        self.ids: list[str] = []

    def save_templates(self) -> None:
        for license in self.licenses:
            license.save_template()

    def fetch_templates(self) -> None:
        for license in self.licenses:
            license.fetch_template()

    def fetch_metadata(self):
        """Find and print license file names and SPDX identifiers"""

        self.driver.get(CHOOSEALICENSE_URL)

        license_link_selector = 'body > div > table > tbody > tr > th[scope="row"] a'
        elements = self.driver.find_elements(By.CSS_SELECTOR, license_link_selector)

        for element in elements:
            license = self._create_template_ref(element)
            self.licenses.append(license)
            self.ids.append(license.spdx_id_lower)

        self.num_licenses = len(self.licenses)

    def generate_manifest(self):
        out_path = os.path.join(self.out_dir, self.manifest_name)
        with open(out_path, "wt", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2)

    def serialize(self):
        return json.dumps(self.data, indent=2)

    def deserialize(self):
        content = self.serialize()
        return json.loads(content)

    @property
    def data(self):
        licenses = []
        for license in self.licenses:
            licenses.append(license.data)

        return {
            "ids": self.ids,
            "licenses": licenses,
        }

    @property
    def manifest_path(self):
        return os.path.join(self.out_dir, self.manifest_name)

    def _create_template_ref(self, web_element) -> LicenseRef:
        name = web_element.text
        spdx_id_lower = web_element.get_attribute("href").split("/").pop()
        license = LicenseRef(name, spdx_id_lower)
        license.template_path = self._build_template_path(spdx_id_lower)
        license.header_path = self._build_template_header_path(spdx_id_lower)
        license.has_header = os.path.isfile(license.header_path)
        return license

    def _build_template_path(self, spdx_id: str):
        filename = "{}.txt".format(spdx_id.lower())
        return os.path.join(self.out_dir, self.templates_dir, filename)

    def _build_template_header_path(self, spdx_id: str):
        filename = "{}.txt".format(spdx_id.lower())
        return os.path.join(self.out_dir, self.headers_dir, filename)


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
