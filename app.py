import database

from datetime import datetime, timedelta
from flask import Flask, jsonify, render_template, request
from flask_mongoengine import MongoEngine

app = Flask(__name__)
app.config.from_pyfile('app.cfg')
db = MongoEngine(app)

@app.route('/api/v1/stats', methods=['POST'])
def submit_stats():
    j = request.get_json()
    stat = database.Statistic(device_id=j['device_hash'],
            model=j['device_name'], version=j['device_version'],
            country=j['device_country'], carrier=j['device_carrier'],
            carrier_id=j['device_carrier_id'])
    stat.save()
    print("Saved")
    return "neat"

@app.route('/api/v1/popular/<string:field>/<int:days>')
def get_devices(field='model', days=90):
    obj = database.get_stats_from(days)
    return jsonify({
        'result': database.get_most_popular(obj, field)
        })


@app.route('/')
def index():
    devices = database.get_stats_from(90)
    total = len(devices)
    return render_template('index.html', total=total, len=len,
            devices=database.get_most_popular(devices, 'model'),
            countries=database.get_most_popular(devices, 'country'))

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
