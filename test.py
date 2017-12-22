from __future__ import unicode_literals

import os
from app import app as main_app
from database import Statistic, Aggregate
import unittest
import json

class StatsTestCase(unittest.TestCase):
    def setUp(self):
        main_app.testing = True
        main_app.config['MONGODB_HOST'] = 'mongomock://loccalhost'
        self.app = main_app.test_client()
    def tearDown(self):
        pass

    def create_statistics(self):
        stats = []
        devices = ['cucumber', 'pumpkin', 'tomato', 'avocado', 'toast']
        for i in range(len(devices)):
            for date in ['20170101', '20170102', '20170103', '20170104', '20170105']:
                for version in ['13.0', '14.1']:
                    for t in ['NIGHTLY', 'UNOFFICIAL']:
                            Statistic(d=str(i), m=devices[i], v='{}-{}-{}-{}'.format(version, date, t, devices[i]), u='US', c='Carrier', c_id='0').save()
                            Aggregate.add_stat(d=str(i), m=devices[i], v='{}-{}-{}-{}'.format(version, date, t, devices[i]), u='US', c='Carrier', c_id='0')

    def test_post(self):
        rv = self.app.post('/api/v1/stats', data=json.dumps(dict(device_hash='1', device_name='cucumber', device_version='14.1-20170101-NIGHTLY-cucumber', device_country='US', device_carrier='Carrier', device_carrier_id='0')), content_type="application/json")
        assert(Statistic.objects().count() == 1)
        Statistic.objects().delete()
        Aggregate.objects().delete()

    def test_popular_stats(self):
        self.create_statistics()
        popular = Aggregate.get_most_popular('model', 90)
        assert(len(popular) == 5)
        Statistic.objects().delete()
        Aggregate.objects().delete()

    def test_field_stats(self):
        self.create_statistics()
        by_field = Aggregate.get_info_by_field("model", "cucumber")
        print(by_field)
        Statistic.objects().delete()
        Aggregate.objects().delete()



if __name__ == "__main__":
    unittest.main()
