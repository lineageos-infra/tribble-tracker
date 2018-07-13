import json
import os
from datetime import datetime
from time import time

import falcon
import jinja2
import mongoengine
from falcon.media.validators import jsonschema
from prometheus_client import multiprocess, generate_latest, CollectorRegistry, CONTENT_TYPE_LATEST, Counter, Histogram

from config import Config as config
from models import Statistic, Aggregate


j2env = jinja2.Environment(
    loader=jinja2.FileSystemLoader("templates"),
    autoescape=jinja2.select_autoescape(['html', 'xml'])
)


REQUEST_LATENCY = Histogram("falcon_request_latency_seconds", "Request Latency", ['method', 'endpoint'])
REQUEST_COUNT = Counter("falcon_request_count", "Request Count", ["method", "endpoint", "status"])

class PrometheusComponent(object):
    def process_request(self, req, resp):
        req.context['start_time'] = time()

    def process_response(self, req, resp, resource, req_suceeded):
        delta = time() - req.context['start_time']
        REQUEST_LATENCY.labels(req.method, req.relative_uri).observe(delta)
        REQUEST_COUNT.labels(req.method, req.relative_uri, resp.status).inc()

class PrometheusMetricsResource(object):
    def on_get(self, req, resp):
        registry = CollectorRegistry()
        multiprocess.MultiProcessCollector(registry)
        resp.body = generate_latest(registry)
        resp.content_type = CONTENT_TYPE_LATEST

def load_template(name):
    return j2env.get_template(name)


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

    def on_get(self, req, resp):
        '''Handles get requests to /api/v1/stats'''
        stats = {
            'model': Aggregate.get_most_popular('model', 90),
            'country': Aggregate.get_most_popular("country", 90),
            'total': Aggregate.get_count(90)
        }
        resp.body = json.dumps(stats)

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
        resp.status = falcon.HTTP_503
        resp.body = "Individual stats are not currently available"
        resp.content_type = "text/html"
        return
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

app = falcon.API(middleware=[PrometheusComponent()])
app.add_route('/', IndexResource())
app.add_route('/{field}/{value}', FieldResource())
app.add_route('/static/{kind}/{filename}', StaticResource())
app.add_route('/api/v1/stats', StatsApiResource())
app.add_route('/metrics', PrometheusMetricsResource())

mongoengine.connect(
    config.MONGODB_DB,
    host=config.MONGODB_HOST,
    port=config.MONGODB_PORT,
    username=config.MONGODB_USERNAME,
    password=config.MONGODB_PASSWORD
)
