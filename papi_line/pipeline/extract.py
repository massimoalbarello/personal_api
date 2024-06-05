import os
import io
import zipfile
import json
from datetime import datetime

USERS_DATALAKE = 'users_datalake'
os.makedirs(USERS_DATALAKE, exist_ok=True)

# A list of MIME types that are associated with ZIP files.
ZIP_MIME_TYPES = ['application/zip', 'application/x-zip', 'application/x-zip-compressed']

def generate_filename(user_id, timestamp, resource):
    return f"{user_id}_{timestamp}_{resource}.json"

def flatten(zf, id):
    filenames = []
    for file_info in zf.infolist():
        if file_info.filename.endswith('MyActivity.json'):
            
            # Generate new filename
            resource = file_info.filename.split('/')[2].lower().replace(' ', '_')
            timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
            new_filename = generate_filename(id, timestamp, resource)
            new_file_path = os.path.join(USERS_DATALAKE, new_filename)
            
            # Read the content of MyActivity.json from the zip file
            with zf.open(file_info) as f:
                content = json.load(f)
            
            # Save the content to the new file
            with open(new_file_path, 'w', encoding='utf-8') as f:
                json.dump(content, f, ensure_ascii=False, indent=4)
            
            filenames.append(new_filename)
            print(f"Saved {new_file_path}")

    return filenames

def unzip_and_flatten(id, url):
    print("👉 Extracting archive...")
    zf = zipfile.ZipFile(io.BytesIO(url.read()), 'r')
    filenames = flatten(zf, id)
    return filenames

def extract(id, url) -> list[str]:
    try:
        content_type = url.headers.get('Content-Type')
        # Check if the Content-Type is one of the ZIP MIME types
        if content_type in ZIP_MIME_TYPES:
            filename = unzip_and_flatten(id, url)
            return filename
        else:
            # TODO: figure out what to do with non-zip files
            content_disposition = url.headers.get('Content-Disposition')
            print(f"👇 File is not a ZIP file:\n{url.headers}")
            content_disposition = url.headers.get('Content-Disposition')
            # extract filename from content-disposition and save the file
            if content_disposition and 'filename=' in content_disposition:
                filename = content_disposition.split('filename=')[-1].strip('\"')
                filename = filename.encode('ISO-8859-1').decode('utf-8')
                print(f"👉 Saving file: {filename}")
                file_path = os.path.join(USERS_DATALAKE, filename)
                with open(file_path, 'wb') as f:
                    f.write(url.read())
                print(f"📁 File {filename} saved successfully")
            else:
                print("❗️ The file from the url is not a zip file and does not have a valid filename.")
    except zipfile.BadZipFile:
        print(f"❗️ The file from the url is not a valid ZIP file or is corrupted.")