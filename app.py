import database

from flask import Flask, request
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

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=80, debug=True)
