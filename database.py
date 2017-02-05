from datetime import datetime, timedelta

from mongoengine import Document
from mongoengine import StringField, DateTimeField

class Statistic(Document):
    d = StringField(required=True)          #device_id
    m = StringField(required=True)          #model
    v = StringField(required=True)          #version
    u = StringField(required=True)          #country
    c = StringField(required=True)          #carrier
    c_id = StringField(required=True)       #carrier_id
    t = DateTimeField(default=datetime.now) #submit_time

    meta = { "indexes": ["m", "u"] }

    @classmethod
    def has_device(cls, device):
        if cls.objects(m=device).first():
            return True
        return False

    @classmethod
    def get_most_popular(cls, field, days):
        #> db.statistic.aggregate({ '$group': {'_id': '$d', 'model': { '$first': '$m'} } }, { '$group': { '_id': '$model', total: { '$sum': 1}}}, {'$sort': {'total': -1}})
        res = cls.objects().aggregate({ '$match': { 't': { '$gte': datetime.now()-timedelta(days=days) } } }, { '$group': {'_id': '$d', field: { '$first': '$' + cls.field_map[field] } } }, { '$group': { '_id': '$' + field, 'total': { '$sum': 1 } }}, {'$sort': {'total': -1} })
        return list(res)

    @classmethod
    def get_count(cls, days=90):
        return cls.objects().aggregate({ '$match': { 't': { '$gte': datetime.now()-timedelta(days=days) } } }, { '$group': { '_id': '$d' } }, { "$group": { "_id": 1, 'count': { '$sum': 1 } } }).next()['count']

    @classmethod
    def get_stats_from(cls, days=90):
        return cls.objects(t__gte=datetime.now()-timedelta(days=days))

    @classmethod
    def get_device(cls, device, days=90):
        #> db.statistic.aggregate({ '$match': { 'm': 'hammerhead'} }, { '$group': {'_id': '$d', 'version': { '$last': '$v'} } }, { '$group': { '_id': '$version', total: { '$sum': 1}}}, {'$sort': {'total': -1}})
        out = {}
        out['version'] = cls.objects().aggregate({ '$match': { 'm': device, 't': { '$gte': datetime.now()-timedelta(days=days) }} }, { '$group': {'_id': '$d', 'version': { '$last': '$v'} } }, { '$group': { '_id': '$version', 'total': { '$sum': 1}}}, {'$sort': {'total': -1}})
        out['country'] = cls.objects().aggregate({ '$match': { 'm': device, 't': { '$gte': datetime.now()-timedelta(days=days) }} }, { '$group': {'_id': '$d', 'country': { '$last': '$u'} } }, { '$group': { '_id': '$country', 'total': { '$sum': 1}}}, {'$sort': {'total': -1}})
        out['carrier'] = cls.objects().aggregate({ '$match': { 'm': device, 't': { '$gte': datetime.now()-timedelta(days=days) }} }, { '$group': {'_id': '$d', 'carrier': { '$last': '$c'} } }, { '$group': { '_id': '$carrier', 'total': { '$sum': 1}}}, {'$sort': {'total': -1}})
        out['total']   = cls.objects().aggregate({ '$match': { 'm': device, 't': { '$gte': datetime.now()-timedelta(days=days) } } }, { '$group': { '_id': '$d' } }, { "$group": { "_id": 1, 'count': { '$sum': 1 } } }).next()['count']
        return out

    field_map = {
        'device_id': 'd',
        'model': 'm',
        'version': 'v',
        'country': 'u',
        'carrier': 'c',
        'carrier_id': 'c_id',
        'submit_time': 't'
    }
