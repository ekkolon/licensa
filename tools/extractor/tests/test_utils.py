import unittest

from utils import extract_license_fields


class TestExtractLicenseFields(unittest.TestCase):
    def setUp(self):
        self.variables = [
            "project",
            "Software Name",
            "projecturl",
            "year",
            "yyyy",
            "Year",
            "fullname",
            "name of copyright owner",
            "name of copyright holder",
            "name of author",
            "email",
        ]
        self.field_map = {
            "project": "project",
            "Software Name": "project",
            "projecturl": "projecturl",
            "year": "year",
            "yyyy": "year",
            "Year": "year",
            "fullname": "fullname",
            "name of copyright owner": "fullname",
            "name of copyright holder": "fullname",
            "name of author": "fullname",
            "email": "email",
        }

    def test_extract_square_bracket_variables(self):
        text = "Copyright (C) [year] [fullname] <email>"

        result = extract_license_fields(text, self.variables, self.field_map)

        self.assertCountEqual(result, ["year", "fullname", "email"])

    def test_extract_angle_bracket_variables(self):
        text = "Copyright (C) <year> <fullname> [email]"

        result = extract_license_fields(text, self.variables, self.field_map)

        self.assertCountEqual(result, ["year", "fullname", "email"])

    def test_extract_mixed_bracket_variables(self):
        text = "Copyright (C) [year] <fullname> [email]"

        result = extract_license_fields(text, self.variables, self.field_map)

        self.assertCountEqual(result, ["year", "fullname", "email"])

    def test_no_matching_variables(self):
        text = "Copyright (C) All rights reserved."

        result = extract_license_fields(text, self.variables, self.field_map)

        self.assertEqual(result, [])

    def test_empty_text(self):
        text = ""

        result = extract_license_fields(text, self.variables, self.field_map)

        self.assertEqual(result, [])

    def test_extract_all_variables(self):
        text = (
            "[project] is developed by [name of author]. Contact [email] for details."
        )

        result = extract_license_fields(text, self.variables, self.field_map)

        self.assertCountEqual(result, ["project", "fullname", "email"])


if __name__ == "__main__":
    unittest.main()
