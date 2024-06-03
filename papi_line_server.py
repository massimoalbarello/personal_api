from flask import Flask, request, jsonify
import urllib
import os
import io
import zipfile
from threading import Thread

NON_ZIP_FILE_DIRECTORY = 'files'
os.makedirs(NON_ZIP_FILE_DIRECTORY, exist_ok=True)

# A list of MIME types that are associated with ZIP files.
ZIP_MIME_TYPES = ['application/zip', 'application/x-zip', 'application/x-zip-compressed']

app = Flask(__name__)

def download_file_background(resource, url) -> None:
    try:
        print(f'👉 Beginning download. {url}')
        url_file = urllib.request.urlopen(url)
        print('👍 Download complete!')
        content_type = url_file.headers.get('Content-Type')
        # Check if the Content-Type is one of the ZIP MIME types
        if content_type in ZIP_MIME_TYPES:
            # Extract the archive.
            print("👉 Extracting archive...")
            zf = zipfile.ZipFile(io.BytesIO(url_file.read()), 'r')
            # Save extracted files in the current directory.
            zf.extractall()
            for f in zf.filelist:
                print(f"📥 File {f.filename} extracted successfully")
        else:
            content_disposition = url_file.headers.get('Content-Disposition')
            print(f"👇 File is not a ZIP file:\n{url_file.headers}")
            content_disposition = url_file.headers.get('Content-Disposition')
            # extract filename from content-disposition and save the file
            if content_disposition and 'filename=' in content_disposition:
                filename = content_disposition.split('filename=')[-1].strip('\"')
                filename = filename.encode('ISO-8859-1').decode('utf-8')
                print(f"👉 Saving file: {filename}")
                file_path = os.path.join(NON_ZIP_FILE_DIRECTORY, filename)
                with open(file_path, 'wb') as f:
                    f.write(url_file.read())
                print(f"📁 File {filename} saved successfully")
            else:
                print("❗️ The file from the url is not a zip file and does not have a valid filename.")
    except zipfile.BadZipFile:
        print(f"❗️ The file from the url is not a valid ZIP file or is corrupted.")
    except Exception as e:
        print("Error: ", e)

@app.route('/download', methods=['POST'])
def download_files():
    print("Received POST request to /download")
    # print("Request JSON payload:", request.json)

    body = request.get_json()
    resource = body.get('resource')
    url = body.get('url')

    if not resource or not url:
        return jsonify({'error': 'Invalid request format'}), 400

    thread = Thread(target=download_file_background, args=(resource, url))
    thread.start()

    return {}, 200

if __name__ == '__main__':
    app.run(port=6969)