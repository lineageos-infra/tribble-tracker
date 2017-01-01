from database import Statistic

from datetime import datetime, timedelta
from flask import Flask, jsonify, render_template, request
from flask_mongoengine import MongoEngine

app = Flask(__name__)
app.config.from_pyfile('app.cfg')
db = MongoEngine(app)

@app.route('/api/v1/stats', methods=['POST'])
def submit_stats():
    j = request.get_json()
    stat = Statistic(d=j['device_hash'],
            m=j['device_name'], v=j['device_version'],
            u=j['device_country'], c=j['device_carrier'],
            c_id=j['device_carrier_id'])
    stat.save()
    print("Saved")
    return "neat"

@app.route('/api/v1/popular/<string:field>/<int:days>')
@app.route('/api/v1/popular/<int:days>')
def get_devices(field='model', days=90):
    obj = Statistic.get_stats_from(days)
    return jsonify({
        'result': Statistic.get_most_popular(obj, field)
        })


@app.route('/')
def index():
    devices = Statistic.get_stats_from(90)
    total = len(devices)
    return render_template('index.html', total=total, len=len,
            devices=Statistic.get_most_popular(devices, 'model'),
            countries=Statistic.get_most_popular(devices, 'country'))

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
