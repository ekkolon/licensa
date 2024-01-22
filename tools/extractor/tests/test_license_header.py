import unittest

from license_header import extract_license_header, invalid_field_error


class TestLicenseHeaderExtraction(unittest.TestCase):
    def test_extract_license_header_valid_input(self):
        content = """\
---
title: BSD Zero Clause License
spdx-id: 0BSD

description: The BSD Zero Clause license goes further than the BSD 2-Clause license to allow you unlimited freedom with the software without requirements to include the copyright notice, license text, or disclaimer in either source or binary forms.

how: Create a text file (typically named LICENSE or LICENSE.txt) in the root of your source code and copy the text of the license into the file.  Replace [year] with the current year and [fullname] with the name (or names) of the copyright holders. You may take the additional step of removing the copyright notice.

using:
  gatsby-starter-default: https://github.com/gatsbyjs/gatsby-starter-default/blob/master/LICENSE
  Toybox: https://github.com/landley/toybox/blob/master/LICENSE
  PickMeUp: https://github.com/nazar-pc/PickMeUp/blob/master/copying.md

permissions:
  - commercial-use
  - distribution
  - modifications
  - private-use

conditions: []

limitations:
  - liability
  - warranty

---

BSD Zero Clause License

Copyright (c) [year] [fullname]

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
"""

        expected_title = "BSD Zero Clause License"
        expected_spdx_id = "0BSD"

        license_header = extract_license_header(content)

        self.assertEqual(license_header.title, expected_title)
        self.assertEqual(license_header.spdx_id, expected_spdx_id)

    def test_extract_license_header_invalid_input(self):
        # Test with content missing SPDX-Identifier
        content = """\
---
title: BSD Zero Clause License
"""

        with self.assertRaises(ValueError) as context:
            extract_license_header(content)

        self.assertEqual(
            str(context.exception), "Failed to determine license header field: spdx_id"
        )

    def test_invalid_field_error(self):
        field_name = "missing_field"
        expected_error_message = (
            "Failed to determine license header field: missing_field"
        )

        error_message = invalid_field_error(field_name)

        self.assertEqual(error_message, expected_error_message)


if __name__ == "__main__":
    unittest.main()
