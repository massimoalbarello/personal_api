from flask import Flask, request, jsonify
import urllib
from threading import Thread
from pipeline.extract import extract
from pipeline.transform import transform

app = Flask(__name__)

def start_etl_pipeline(id, url) -> None:
    try:
        print(f'👉 Beginning download. {url}')
        url_file = urllib.request.urlopen(url)
        print('👍 Download complete!')
        filenames = extract(id, url_file)
        transform(filenames)

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

    thread = Thread(target=start_etl_pipeline, args=(id, url))
    thread.start()

    return {}, 200

if __name__ == '__main__':
    app.run(port=6969)