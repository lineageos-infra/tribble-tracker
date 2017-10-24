import os

class Config(object):
    MONGODB_DB = os.environ.get("MONGODB_DB", "stats")
    MONGODB_USERNAME = os.environ.get("MONGODB_USERNAME", "")
    MONGODB_PASSWORD = os.environ.get("MONGODB_PASSWORD", "")
    MONGODB_HOST = os.environ.get("MONGODB_HOST", "127.0.0.1")
    MONGODB_PORT = int(os.environ.get("MONGODB_PORT", "27017"))

    REDIS_URL = os.environ.get("REDIS_URL", "redis://localhost")
