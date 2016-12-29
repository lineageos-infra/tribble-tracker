from datetime import datetime

from mongoengine import Document
from mongoengine import StringField, DateTimeField

class Statistic(Document):
    device_id = StringField(required=True)
    model = StringField(required=True)
    version = StringField(required=True)
    country = StringField(required=True)
    carrier = StringField(required=True)
    carrier_id = StringField(required=True)
    submit_time = DateTimeField(default=datetime.now)


