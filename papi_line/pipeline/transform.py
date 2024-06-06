import polars as pl
import os
import json

from pipeline.extract import USERS_DATALAKE
from utils import generate_filename

MERGED = USERS_DATALAKE + '/merged'
os.makedirs(MERGED, exist_ok=True)


MY_ACTIVITY_FIELDS = {
    'header': pl.String,              # The card title that will typically be either an app name, domain name, or product name.
    'title': pl.String,               # High level summary of the user activity.
    'titleUrl': pl.String,            # Url used in the title.
    'subtitles': pl.List(pl.Struct({'name': pl.String})),           # Detailed user activity shown underneath the title.
    'description': pl.String,         # Extra information that helps explain the activity taken by the user.
    'time': pl.String,                # Time and date the user did the activity.
    # 'products': pl.List(pl.String),   # The Products that this data is a part of.
    'details': pl.List(pl.Struct({'name': pl.String})),             # Used to display information about where the user activity comes from, such as if the user activity was the result of the user interacting with an ad or app.
    # 'activityControls': pl.List,    # Shows the Google data collection settings (like Location History and Web & App Activity) that save data in a user’s Google account that the exported data is part of.
    'locationInfos': pl.List(pl.Struct({'name': pl.String, 'url': pl.String, 'source': pl.String, 'sourceUrl': pl.String})),       # The location(s) associated with this activity.
    # 'imageFile': pl.String,           # The local file name for the image attachment.
    # 'audioFiles': pl.List,          # The list of local file names for the audio attachments.
    # 'attachedFiles': pl.List        # The list of local file names for the attached files.
}

def transform(user_id, filenames):
    print("👉 Transforming events in", filenames)
    dataframes = []
    for filename in filenames:
        if filename.endswith('.json'):
            file_path = os.path.join(USERS_DATALAKE, filename)
            # Read the JSON file into a Polars DataFrame
            df = pl.read_json(file_path)

            # Convert columns to correct data types
            for col, dtype in MY_ACTIVITY_FIELDS.items():
                if col in df.columns:
                    df = df.with_columns(df[col].cast(dtype))
                else:
                    # Add a missing column with null values
                    df = df.with_columns(pl.lit(None).cast(dtype).alias(col))

            # Reorder columns
            df = df.select(sorted(MY_ACTIVITY_FIELDS.keys()))

            dataframes.append(df)

    # Merge dataframes and sort by time
    df_concat = pl.concat(dataframes).sort(by='time', descending=True)
    print(df_concat)

    # Save merged file
    resources = [filename.split('_')[3].split('.json')[0] for filename in filenames]
    new_file_path = os.path.join(MERGED, generate_filename(user_id, '_'.join(resources) + '_merged'))
    with open(new_file_path, 'w', encoding='utf-8') as f:
        json.dump(df_concat.to_dicts(), f, ensure_ascii=False, indent=4)