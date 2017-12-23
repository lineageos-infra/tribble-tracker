import os
from datetime import datetime

import falcon
from falcon.media.validators import jsonschema
import jinja2
import mongoengine

from config import Config as config
from models import Statistic, Aggregate

def load_template(name):
    path = os.path.join('templates', name)
    with open(os.path.abspath(path), 'r') as f:
        return jinja2.Template(f.read())


class StatsApiResource(object):

    schema = {
        'title': 'Stats Object',
        'type': 'object',
        'properties': {
            'device_hash': {
                'type': 'string'
            },
            'device_name': {
                'type': 'string'
            },
            'device_version': {
                'type': 'string'
            },
            'device_country': {
                'type': 'string'
            },
            'device_carrier': {
                'type': 'string'
            },
            'device_carrier_id': {
                'type': 'string'
            },
        },
        'required': ['device_hash', 'device_name', 'device_version', 'device_country', 'device_carrier', 'device_carrier_id']
    }
    @jsonschema.validate(schema)
    def on_post(self, req, resp):
        '''Handles post requests to /api/v1/stats'''
        data = req.media
        Aggregate.add_stat(d=data['device_hash'],
                           m=data['device_name'], v=data['device_version'],
                           u=data['device_country'], c=data['device_carrier'],
                           c_id=data['device_carrier_id'])
        resp.body = "neat"
        resp.content_type = "text/plain"


class StaticResource(object):
    def on_get(self, req, resp, kind, filename):
        if kind == "css":
            resp.content_type = "text/css"
        resp.stream = open(os.path.abspath(os.path.join('static', kind, filename)), 'rb')


class IndexResource(object):
    def on_get(self, req, resp):
        '''Render the main page'''
        stats = {"model": Aggregate.get_most_popular('model', 90), "country": Aggregate.get_most_popular("country", 90), "total": Aggregate.get_count(90)}
        template = load_template('index.html').render(stats=stats, columns=["model", "country"], date=datetime.utcnow().strftime("%Y-%m-%d %H:%M"))
        resp.content_type = 'text/html'
        resp.body = template

class FieldResource(object):
    def on_get(self, req, resp, field, value):
        if not field in Aggregate.field_map.keys():
            resp.status = falcon.HTTP_404
            resp.content_type = "text/plain"
            resp.body = "Not Found"
            return
        valuemap = {'model': ['version', 'country'], 'carrier': ['model', 'country'], 'version': ['model', 'country'], 'country': ['model', 'carrier']}
        stats = Aggregate.get_info_by_field(field, value, left=valuemap[field][0], right=valuemap[field][1])
        template = load_template('index.html').render(stats=stats, columns=valuemap[field], value=value, date=datetime.utcnow().strftime("%Y-%m-%d %H:%M"))
        resp.content_type = "text/html"
        resp.body = template

app = falcon.API()
app.add_route('/', IndexResource())
app.add_route('/{field}/{value}', FieldResource())
app.add_route('/static/{kind}/{filename}', StaticResource())
app.add_route('/api/v1/stats', StatsApiResource())

mongoengine.connect(
    config.MONGODB_DB,
    host=config.MONGODB_HOST,
    port=config.MONGODB_PORT,
    username=config.MONGODB_USERNAME,
    password=config.MONGODB_PASSWORD
)
