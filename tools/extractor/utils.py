import re

VARIABLES = [
    "project",
    "Software Name",
    "projecturl",
    "year",
    "yyyy",
    "Year",
    "fullname",
    "name of copyright owner",
    "name of copyright holder",
    "name of author" "email",
]

FIELD_MAP = {
    # Project
    "project": "project",
    "Software Name": "project",
    "projecturl": "projecturl",
    # License year
    "year": "year",
    "yyyy": "year",
    "Year": "year",
    # Copyright owner
    "fullname": "fullname",
    "name of copyright owner": "fullname",
    "name of copyright holder": "fullname",
    "name of author": "fullname",
    "email": "email",
}


def extract_license_fields(text, variables=VARIABLES, field_map=FIELD_MAP):
    """
    Extracts interpolatable string from the given text.

    Parameters:
        text (str): The input text containing license variables.
        variables (list): List of variable names to search for.
        field_map (dict): Mapping of variable names to their respective fields.

    Returns:
        list: A list of unique variable names found in the text.
    """
    if variables is None:
        variables = VARIABLES

    if field_map is None:
        field_map = FIELD_MAP

    # Use regular expression to find interpolatable strings
    matches = re.finditer(r"\[([^\[\]]+)\]|\<([^\<\>]+)\>", text)

    # Extract matched variable names
    extracted_variables = set(
        match.group(1) or match.group(2)
        for match in matches
        if match.group(1) or match.group(2)
    )

    fields = []

    # Replace variable names with their respective fields
    for var in extracted_variables:
        field = field_map.get(var)
        if field:
            fields.append(field)

    return fields
