# run_tests.py

import unittest
from tests.test_license_header import TestLicenseHeaderExtraction

if __name__ == "__main__":
    unittest.TextTestRunner().run(
        unittest.TestLoader().loadTestsFromTestCase(TestLicenseHeaderExtraction)
    )
