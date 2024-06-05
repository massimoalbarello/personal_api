import unittest

from pipeline.transform import transform

TEST_FILES = [
    "123_20240605_152739_discover.json",
    "123_20240605_152739_google_lens.json",
    "123_20240605_152739_image_search.json",
    "123_20240605_152739_search.json",
    "123_20240605_152739_video_search.json",
]

class TestTransformFunction(unittest.TestCase):
    def test_transform(self):
        transform(TEST_FILES)

        self.assertTrue(True)

if __name__ == '__main__':
    unittest.main()