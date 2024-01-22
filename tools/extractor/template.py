import json
import requests

# from selenium.webdriver.chrome.webdriver import WebDriver
from selenium.webdriver.common.by import By

GITHUB_LICENSES_URL_BASE = (
    "https://raw.githubusercontent.com/github/choosealicense.com/gh-pages/_licenses"
)

CHOOSEALICENSE_URL = "https://choosealicense.com/appendix/"


class TemplateRef:
    def __init__(self, title: str, spdx_id_lower: str):
        self.title = title
        self.spdx_id_lower = spdx_id_lower
        self.template_url = self._generate_template_url()
        self.template: str = None
        self.has_header = False

    def fetch_template(self):
        res = requests.get(self.template_url)
        self.template = res.text
        self._extract_meta()
        return self.template

    def save_template(self, path: str):
        lines = self.template.splitlines(True)
        # Save template file at specified template_path
        # Note: encoding kwarg is required
        with open(path, "wt", encoding="utf-8") as f:
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
        self.title = (
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


class TemplateRefStore:
    def __init__(self, driver):
        self.driver = driver
        self.licenses: list[TemplateRef] = []

    def fetch_available_licenses(self):
        """Find and print license file names and SPDX identifiers"""

        # List to store license information in the form of dictionaries
        licenses = []

        # Open the Choose a License GitHub page in the Chrome browser
        self.driver.get(CHOOSEALICENSE_URL)

        # Find all elements matching the specified CSS selector
        elements = self.driver.find_elements(
            By.CSS_SELECTOR, 'body > div > table > tbody > tr > th[scope="row"] a'
        )

        # Iterate through each element and extract title and SPDX identifier
        for element in elements:
            title = element.text
            spdx_id_lower = element.get_attribute("href").split("/").pop()
            license = TemplateRef(title, spdx_id_lower)
            self.licenses.append(license)

    def serialize(self):
        return json.dumps(self.licenses, indent=2)

    def deserialize(self):
        content = self.serialize()
        return json.loads(content)

    @property
    def num_refs(self):
        len(self.licenses)


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


def extract_license_meta(template: str):
    """
    Extracts license metadata from a multiline string.

    Parameters:
        content (str): Multiline string containing license metadata.

    Returns:
        dict[str, str]: Extracted license metadata.
    """
    lines = template.splitlines()

    # Extract the title
    title_line = next((line for line in lines if line.startswith("title:")), None)
    title = (
        title_line.split(": ", 1)[1].strip()
        if title_line
        else _invalid_field_error("title")
    )

    # Extract the SPDX-Identifier
    spdx_id_line = next((line for line in lines if line.startswith("spdx-id:")), None)
    spdx_id = (
        spdx_id_line.split(": ", 1)[1].strip()
        if spdx_id_line
        else _invalid_field_error("spdx_id")
    )

    return {"title": title, "spdx_id": spdx_id}


def _invalid_field_error(field: str) -> str:
    """
    Generates an error message for an invalid field.

    Parameters:
        field (str): The invalid field name.

    Returns:
        str: Error message.
    """
    return f"Failed to determine license header field: {field}"
