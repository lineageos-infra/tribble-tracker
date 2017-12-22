from __future__ import unicode_literals
from database import Aggregate, Statistic

from datetime import datetime, timedelta
from flask import Flask, jsonify, render_template, request, abort
from flask_mongoengine import MongoEngine
from flask_redis import FlaskRedis
from prometheus_client import multiprocess, generate_latest, CollectorRegistry, CONTENT_TYPE_LATEST, Counter, Histogram

import ast
import click
import codecs
import hashlib
import random
import string

app = Flask(__name__)
app.config.from_object("config.Config")
db = MongoEngine(app)
redis_cache = FlaskRedis(app)

##########################
# Flask Commands
##########################

@app.cli.command()
@click.argument("start")
@click.argument("end")
@click.argument("filename")
@click.argument("echo", default=10000)
def dump_json(start, end, filename, echo=10000):
    a = datetime(*map(int, start.split("-")))
    b = datetime(*map(int, end.split("-")))
    counter = 0
    salt = ''.join(random.choice(string.ascii_uppercase + string.digits) for _ in range(200))
    with codecs.open(filename, 'w', 'utf-8') as f:
        f.write("[\n")
        for i in Statistic.objects(t__gte=a, t__lt=b).no_cache():
            device_id = hashlib.sha256(salt.encode() + i.d.encode()).hexdigest().upper()
            f.write(u'{{"d": "{d}", "t": "{t}", "m": "{m}", "v": "{v}", "u": "{u}"}},\n'.format(d=device_id, t=i.t.strftime("%Y%m%d %H%M"), m=i.m, v=i.v, u=i.u))
            counter += 1
            if counter % echo == 0:
                print(counter)
        f.write("]")

@app.cli.command()
def generate_caches():
    #main page
    print("Generating cache/main")
    stats = { "model": Aggregate.get_most_popular('model', 90), "country": Aggregate.get_most_popular("country", 90), "total": Aggregate.get_count(90)}
    template = render_template('index.html', stats=stats, columns=["model", "country"], date=datetime.utcnow().strftime("%Y-%m-%d %H:%M"))
    redis_cache.set("cache/main", template)
    #field pages
    for field in ['model', 'carrier', 'version', 'country']:
        print("Generating cache/popular/{}".format(field))
        redis_cache.set("cache/popular/{}".format(field), Aggregate.get_most_popular(field, 90))
        for item in ast.literal_eval(redis_cache.get("cache/popular/{}".format(field))):
            if item["total"] < 1000:
                continue
            value = item["_id"]
            print("Generating cache/{}/{}".format(field, value))
            try:
                redis_cache.set("cache/{}/{}".format(field, value), Aggregate.get_info_by_field(field, value))
            except Exception as e:
                print(e.message)

##########################
# Metrics!
##########################

REQUEST_LATENCY = Histogram("flask_request_latency_seconds", "Request Latency", ['method', 'endpoint'])
REQUEST_COUNT = Counter("flask_request_count", "Request Count", ["method", "endpoint", "status"])

@app.before_request
def start_timer():
    request.stats_start = time()

@app.after_request
def stop_timer(response):
    delta = time() - request.stats_start
    REQUEST_LATENCY.labels(request.method, request.endpoint).observe(delta) #pylint: disable=no-member
    REQUEST_COUNT.labels(request.method, request.endpoint, response.status_code).inc() #pylint: disable=no-member
    return response

@app.route('/metrics')
def metrics():
    registry = CollectorRegistry()
    multiprocess.MultiProcessCollector(registry)
    return Response(generate_latest(registry), mimetype=CONTENT_TYPE_LATEST)

##########################
# API
##########################

@app.route('/api/v1/stats', methods=['POST'])
def submit_stats():
    j = request.get_json()
    Aggregate.add_stat(d=j['device_hash'],
                       m=j['device_name'], v=j['device_version'],
                       u=j['device_country'], c=j['device_carrier'],
                       c_id=j['device_carrier_id'])
    return "", 200

@app.route('/api/v1/popular/<int:days>')
def get_devices(field='model'):
    if field == 'device_id':
        return jsonify({'result': 'No!'})
    cached = redis_cache.get("cache/popular/{}".format(field))
    if not cached:
        return jsonify({})
    else:
        return jsonify({'result': ast.literal_eval(cached)})


@app.route('/api/v1/<string:field>/<string:value>')
def api_stats_by_field(field, value):
    '''Get stats by a specific field. Examples:
       /model/hammerhead
       /country/in
       /carrier/T-Mobile
       Each thing returns json blob'''
    cached = redis_cache.get("cache/{}/{}".format(field, value))
    if not cached:
        return jsonify({})
    else:
        return jsonify(ast.literal_eval(cached))

##########################
# Web Views
##########################

@app.route('/')
def index():
    return redis_cache.get("cache/main") or "This page isn't rendered yet"

@app.route('/<string:field>/<string:value>/')
def stats_by_field(field, value):
    key = "cache/{}/{}".format(field, value)
    cached = redis_cache.get(key)
    if not cached:
        return "This page isn't rendered yet"
    valuemap = { 'model': ['version', 'country'], 'carrier': ['model', 'country'], 'version': ['model', 'country'], 'country': ['model', 'carrier'] }
    return render_template("index.html", stats=ast.literal_eval(cached), columns=valuemap[field], value=value)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
