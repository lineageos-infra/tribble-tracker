from database import Aggregate, Statistic

from datetime import datetime, timedelta
from flask import Flask, jsonify, render_template, request, abort
from flask_mongoengine import MongoEngine
from flask_caching import Cache

import click
import codecs
import hashlib
import random
import string

app = Flask(__name__)
app.config.from_pyfile('app.cfg')
db = MongoEngine(app)
cache = Cache(app)

force_cache_update = lambda: False

@app.cli.command()
def generate_caches():
    global force_cache_update
    force_cache_update = lambda: True
    get_most_popular("model", 90)
    get_most_popular("country", 90)
    get_count(90)

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

@app.route('/api/v1/stats', methods=['POST'])
def submit_stats():
    j = request.get_json()
    Aggregate.add_stat(d=j['device_hash'],
                       m=j['device_name'], v=j['device_version'],
                       u=j['device_country'], c=j['device_carrier'],
                       c_id=j['device_carrier_id'])
    return "", 200

@app.route('/api/v1/popular/<string:field>/<int:days>')
@app.route('/api/v1/popular/<int:days>')
@cache.cached(timeout=3600)
def get_devices(field='model', days=90):
    if app.config['COLLECT_ONLY']:
        return ""
    if field == 'device_id':
        return jsonify({'result': 'No!'})
    return jsonify({
        'result': get_most_popular(field, days)
        })

@app.route('/')
@cache.cached(timeout=3600)
def index():
    if app.config['COLLECT_ONLY']:
        return "Stats are currently being collected, but we're unable to display them due to load"
    stats = { "model": get_most_popular('model', 90), "country": get_most_popular("country", 90), "total": get_count(90), "official": get_official_count(90)}
    return render_template('index.html', stats=stats, columns=["model", "country"])

@app.route('/api/v1/<string:field>/<string:value>')
@cache.cached(timeout=3600)
def api_stats_by_field(field, value):
    '''Get stats by a specific field. Examples:
       /model/hammerhead
       /country/in
       /carrier/T-Mobile
       Each thing returns json blob'''
    if app.config['COLLECT_ONLY']:
        return ""
    return jsonify(get_info_by_field(field, value))

@app.route('/<string:field>/<string:value>/')
@cache.cached(timeout=3600)
def stats_by_field(field, value):
    if app.config['COLLECT_ONLY']:
        return ""
    valuemap = { 'model': ['version', 'country'], 'carrier': ['model', 'country'], 'version': ['model', 'country'], 'country': ['model', 'carrier'] }

    if not field in ['model', 'carrier', 'version', 'country'] or not has_thing(field, value): 
        abort(404)

    stats = get_info_by_field(field, value)
    return render_template("index.html", stats=stats, columns=valuemap[field], value=value)

#More caches!

@cache.memoize(forced_update=force_cache_update)
def get_most_popular(thing, count):
    return Aggregate.get_most_popular(thing, count)

@cache.memoize(forced_update=force_cache_update)
def get_official_count(count):
    return Aggregate.get_official_count(count)

@cache.memoize(forced_update=force_cache_update)
def get_count(count):
    return Aggregate.get_count(count)

@cache.memoize(forced_update=force_cache_update)
def get_info_by_field(field, value):
    return Aggregate.get_info_by_field(field, value)

@cache.memoize(forced_update=force_cache_update)
def has_thing(field, value):
    return Aggregate.has_thing(field, value)


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
