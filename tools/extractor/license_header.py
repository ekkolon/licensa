import json


class LicenseHeader:
    def __init__(self, title: str, spdx_id: str):
        self.title = title
        self.spdx_id = spdx_id

    def json(self):
        return json.dumps({"title": self.title, "spdx_id": self.spdx_id}, indent=2)


def extract_license_header(content: str) -> LicenseHeader:
    """
    Extracts license metadata from a multiline string.

    Parameters:
        content (str): Multiline string containing license metadata.

    Returns:
        LicenseHeader: Extracted license metadata.
    """
    lines = content.splitlines()

    # Extract the title
    title_line = next((line for line in lines if line.startswith("title:")), None)
    title = (
        title_line.split(": ", 1)[1].strip()
        if title_line
        else invalid_field_error("title")
    )

    # Extract the SPDX-Identifier
    spdx_id_line = next((line for line in lines if line.startswith("spdx-id:")), None)
    spdx_id = (
        spdx_id_line.split(": ", 1)[1].strip()
        if spdx_id_line
        else invalid_field_error("spdx_id")
    )

    return LicenseHeader(title, spdx_id)


def invalid_field_error(field: str) -> str:
    """
    Generates an error message for an invalid field.

    Parameters:
        field (str): The invalid field name.

    Returns:
        str: Error message.
    """
    return f"Failed to determine license header field: {field}"
