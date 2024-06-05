import unittest
from unittest.mock import MagicMock, mock_open, patch, ANY
import zipfile
import json
from io import BytesIO
from pipeline.extract import flatten

class TestFlattenFunction(unittest.TestCase):
    @patch('builtins.open', new_callable=mock_open)
    @patch('json.dump')
    @patch('zipfile.ZipFile')
    def test_flatten(self, mock_zipfile, mock_json_dump, mock_file):
        # Prepare mock zip file
        mock_zip = MagicMock()
        mock_zipfile.return_value = mock_zip
        mock_file_info = zipfile.ZipInfo(filename='Portability/My Activity/Video Search/MyActivity.json')
        mock_zip.infolist.return_value = [mock_file_info]
        
        # Mock the content of MyActivity.json
        mock_json_content = {'key': 'value'}
        mock_zip.open.return_value = BytesIO(json.dumps(mock_json_content).encode('utf-8'))
        
        # Mock datetime to return a fixed timestamp
        with patch('datetime.datetime') as mock_datetime:
            mock_datetime.now.return_value.strftime.return_value = '20240101_000000'
            
            # Call the flatten function
            flatten(mock_zip, 'test_user')
            
            # Check if the file was written with the correct name and content
            # Use ANY for the part of the filename that changes
            mock_file.assert_called_once_with(ANY, 'w', encoding='utf-8')
            # Verify the filename part that does not change
            filename_arg = mock_file.call_args[0][0]
            self.assertTrue(filename_arg.startswith('users_data/test_user_'))
            self.assertTrue(filename_arg.endswith('_video_search.json'))
            mock_json_dump.assert_called_once_with(mock_json_content, mock_file(), ensure_ascii=False, indent=4)
            mock_zip.open.assert_called_once_with(mock_file_info)

if __name__ == '__main__':
    unittest.main()
