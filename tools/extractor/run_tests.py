# run_tests.py

import unittest

from tests.test_utils import TestExtractLicenseFields

if __name__ == "__main__":
    unittest.TextTestRunner().run(
        unittest.TestLoader().loadTestsFromTestCase(TestExtractLicenseFields)
    )
