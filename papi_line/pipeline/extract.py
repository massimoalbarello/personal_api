import os
import io
import zipfile


NON_ZIP_FILE_DIRECTORY = 'files'
os.makedirs(NON_ZIP_FILE_DIRECTORY, exist_ok=True)

# A list of MIME types that are associated with ZIP files.
ZIP_MIME_TYPES = ['application/zip', 'application/x-zip', 'application/x-zip-compressed']

def extract_file(url):
    try:
        content_type = url.headers.get('Content-Type')
        # Check if the Content-Type is one of the ZIP MIME types
        if content_type in ZIP_MIME_TYPES:
            # Extract the archive.
            print("👉 Extracting archive...")
            zf = zipfile.ZipFile(io.BytesIO(url.read()), 'r')
            # Save extracted files in the current directory.
            zf.extractall()
            for f in zf.filelist:
                print(f"📥 File {f.filename} extracted successfully")
        else:
            content_disposition = url.headers.get('Content-Disposition')
            print(f"👇 File is not a ZIP file:\n{url.headers}")
            content_disposition = url.headers.get('Content-Disposition')
            # extract filename from content-disposition and save the file
            if content_disposition and 'filename=' in content_disposition:
                filename = content_disposition.split('filename=')[-1].strip('\"')
                filename = filename.encode('ISO-8859-1').decode('utf-8')
                print(f"👉 Saving file: {filename}")
                file_path = os.path.join(NON_ZIP_FILE_DIRECTORY, filename)
                with open(file_path, 'wb') as f:
                    f.write(url.read())
                print(f"📁 File {filename} saved successfully")
            else:
                print("❗️ The file from the url is not a zip file and does not have a valid filename.")
    except zipfile.BadZipFile:
        print(f"❗️ The file from the url is not a valid ZIP file or is corrupted.")