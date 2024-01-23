import json
import os

import requests
from selenium.webdriver.chrome.webdriver import WebDriver
from selenium.webdriver.common.by import By

GITHUB_LICENSES_URL_BASE = (
    "https://raw.githubusercontent.com/github/choosealicense.com/gh-pages/_licenses"
)

CHOOSEALICENSE_URL = "https://choosealicense.com/appendix/"


class LicenseRef:
    def __init__(self, name: str, spdx_id_lower: str):
        """
        Represents a license reference with metadata and template information.

        Parameters:
            name (str): The name of the license.
            spdx_id_lower (str): The SPDX identifier in lowercase.

        Attributes:
            name (str): The name of the license.
            spdx_id_lower (str): The SPDX identifier in lowercase.
            spdx_id (str): The SPDX identifier.
            has_header (bool): Indicates if the license has a header.
            nickname (str): The nickname of the license.
            fields (list): List of fields extracted from the license template.
            template (str): The content of the license template.
            template_path (str): The local path where the template is stored.
            template_url (str): The URL to fetch the license template.
            header_path (str): The local path where the license header is stored.
        """
        self.name = name
        self.spdx_id_lower = spdx_id_lower
        self.spdx_id = None
        self.has_header = False
        self.nickname = None

        self.template: str = None
        self.template_path = None
        self.template_url = self._generate_template_url()
        self.header_path = None

    def fetch_template(self):
        """
        Fetches the license template content from the template URL.

        Returns:
            str: The content of the license template.
        """
        res = requests.get(self.template_url)
        self.template = res.text
        self._extract_meta()
        return self.template

    @property
    def data(self):
        """
        Returns a dictionary representation of the LicenseRef instance.

        Returns:
            dict: LicenseRef data.
        """
        return {
            "name": self.name,
            "spdxId": self.spdx_id,
            "spdxIdLower": self.spdx_id_lower,
            "nickname": self.nickname,
            "hasHeader": self.has_header,
            "templateUrl": self.template_url,
        }

    def save_template(self):
        """
        Saves the license template to the specified template_path.

        Raises:
            ValueError: If the template is empty or the template_path is not valid.
        """
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

        Raises:
            ValueError: If the template is accessed before it has been fetched.
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
        """
        Generates the URL to fetch the license template.

        Returns:
            str: The URL to fetch the license template.
        """
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
        """
        Manages the storage and retrieval of license templates and metadata.

        Parameters:
            driver (WebDriver): The Selenium WebDriver instance.
            out_dir (str): The output directory for storing templates and metadata.
            manifest_name (str): The name of the manifest file.
            templates_dir (str): The directory for storing license templates.
            headers_dir (str): The directory for storing license headers.
        """
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
        """Saves all license templates to their respective template paths."""
        for license in self.licenses:
            license.save_template()

    def fetch_templates(self) -> None:
        """Fetches all license templates."""
        for license in self.licenses:
            license.fetch_template()

    def fetch_metadata(self):
        """Finds and prints license file names and SPDX identifiers."""
        self.driver.get(CHOOSEALICENSE_URL)

        license_link_selector = 'body > div > table > tbody > tr > th[scope="row"] a'
        elements = self.driver.find_elements(By.CSS_SELECTOR, license_link_selector)

        for element in elements:
            license = self._create_template_ref(element)
            self.licenses.append(license)
            self.ids.append(license.spdx_id_lower)

        self.num_licenses = len(self.licenses)

    def generate_manifest(self):
        """Generates a manifest file containing license metadata."""
        out_path = os.path.join(self.out_dir, self.manifest_name)
        with open(out_path, "wt", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2)
            f.close()

    def serialize(self):
        """Serializes license metadata to a JSON-formatted string."""
        return json.dumps(self.data, indent=2)

    def deserialize(self):
        """Deserializes license metadata from a JSON-formatted string."""
        content = self.serialize()
        return json.loads(content)

    @property
    def data(self):
        """
        Returns a dictionary representation of the LicenseStore instance.

        Returns:
            dict: LicenseStore data.
        """
        licenses = []
        for license in self.licenses:
            licenses.append(license.data)

        return {
            "ids": self.ids,
            "licenses": licenses,
        }

    @property
    def manifest_path(self):
        """
        Returns the path to the manifest file.

        Returns:
            str: The path to the manifest file.
        """
        return os.path.join(self.out_dir, self.manifest_name)

    def _create_template_ref(self, web_element) -> LicenseRef:
        """
        Creates a LicenseRef instance from a Selenium WebElement.

        Parameters:
            web_element (WebElement): The Selenium WebElement.

        Returns:
            LicenseRef: The created LicenseRef instance.
        """
        name = web_element.text
        spdx_id_lower = web_element.get_attribute("href").split("/").pop()
        license = LicenseRef(name, spdx_id_lower)
        license.template_path = self._build_template_path(spdx_id_lower)
        license.header_path = self._build_template_header_path(spdx_id_lower)
        license.has_header = os.path.isfile(license.header_path)
        return license

    def _build_template_path(self, spdx_id: str):
        """
        Builds the path to a license template file.

        Parameters:
            spdx_id (str): The SPDX identifier.

        Returns:
            str: The path to the license template file.
        """
        filename = "{}.txt".format(spdx_id.lower())
        return os.path.join(self.out_dir, self.templates_dir, filename)

    def _build_template_header_path(self, spdx_id: str):
        """
        Builds the path to a license header file.

        Parameters:
            spdx_id (str): The SPDX identifier.

        Returns:
            str: The path to the license header file.
        """
        filename = "{}.txt".format(spdx_id.lower())
        return os.path.join(self.out_dir, self.headers_dir, filename)
