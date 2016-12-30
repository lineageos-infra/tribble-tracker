from datetime import datetime, timedelta

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


def get_most_popular(objects, field):
    res = objects.aggregate({ '$group': { '_id': '$' + field, 'total': { '$sum': 1 } } })
    return sorted(list(res), key=lambda a: a['total'], reverse=True)

def get_stats_from(days=90):
    return Statistic.objects(submit_time__gte=datetime.now()-timedelta(days=days))
