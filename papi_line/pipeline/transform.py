import polars as pl
import os
import json

from pipeline.extract import USERS_DATALAKE

FILTERED = USERS_DATALAKE + '/filtered'
os.makedirs(FILTERED, exist_ok=True)


MY_ACTIVITY_FIELDS = [
    'header',           # The card title that will typically be either an app name, domain name, or product name.
    'title',            # High level summary of the user activity.
    'titleUrl',         # Url used in the title.
    'subtitles',        # Detailed user activity shown underneath the title.
    'description',      # Extra information that helps explain the activity taken by the user.
    'time',             # Time and date the user did the activity.
    # 'products',         # The Products that this data is a part of.
    'details',          # Used to display information about where the user activity comes from, such as if the user activity was the result of the user interacting with an ad or app.
    # 'activityControls', # Shows the Google data collection settings (like Location History and Web & App Activity) that save data in a user’s Google account that the exported data is part of.
    'locationInfos',    # The location(s) associated with this activity.
    # 'imageFile',        # The local file name for the image attachment.
    # 'audioFiles',       # The list of local file names for the audio attachments.
    # 'attachedFiles'     # The list of local file names for the attached files.
]

def transform(filenames):
    for filename in filenames:
        if filename.endswith('.json'):
            file_path = os.path.join(USERS_DATALAKE, filename)
            # Read the JSON file into a Polars DataFrame
            df = pl.read_json(file_path)

            # Check for missing fields and select only available ones
            available_fields = [field for field in MY_ACTIVITY_FIELDS if field in df.columns]
            if available_fields:
                df = df.select(available_fields)
            print(df)

            data = df.to_dicts()

            # Write each dictionary as a separate JSON object in the output file
            new_file_path = os.path.join(USERS_DATALAKE, 'filtered', filename.replace('.json', '_my_activity.json'))
            with open(new_file_path, 'w', encoding='utf-8') as f:
                json.dump(data, f, ensure_ascii=False, indent=4)