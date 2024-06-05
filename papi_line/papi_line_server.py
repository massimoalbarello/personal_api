from flask import Flask, request, jsonify
import urllib
from threading import Thread
from pipeline.extract import extract_file

app = Flask(__name__)

def download_file_background(id, resource, url) -> None:
    try:
        print(f'👉 Beginning download. {url}')
        url_file = urllib.request.urlopen(url)
        print('👍 Download complete!')
        extract_file(url_file)
    except Exception as e:
        print("Error: ", e)

@app.route('/download', methods=['POST'])
def download_files():
    print("Received POST request to /download")
    # print("Request JSON payload:", request.json)

    body = request.get_json()
    id = body.get('id')
    resource = body.get('resource')
    url = body.get('url')

    if not id or not resource or not url:
        return jsonify({'error': 'Invalid request format'}), 400

    thread = Thread(target=download_file_background, args=(id, resource, url))
    thread.start()

    return {}, 200

if __name__ == '__main__':
    app.run(port=6969)