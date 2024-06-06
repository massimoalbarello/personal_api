from datetime import datetime

def generate_filename(user_id, suffix):
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    return f"{user_id}_{timestamp}_{suffix}.json"