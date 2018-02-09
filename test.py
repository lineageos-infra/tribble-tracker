from __future__ import unicode_literals

import os
import pytest
import mongoengine

from falcon import testing
from app import app
from models import Statistic, Aggregate
import unittest
import json

@pytest.fixture()
def client():
    mongoengine.connection.disconnect()
    mongoengine.connect("testdb", host="mongomock://localhost")
    return testing.TestClient(app)

def create_statistics():
    stats = []
    devices = ['cucumber', 'pumpkin', 'tomato', 'avocado', 'toast']
    for i in range(len(devices)):
        for date in ['20170101', '20170102', '20170103', '20170104', '20170105']:
            for version in ['13.0', '14.1']:
                for t in ['NIGHTLY', 'UNOFFICIAL']:
                        Statistic(d=str(i), m=devices[i], v='{}-{}-{}-{}'.format(version, date, t, devices[i]), u='US', c='Carrier', c_id='0').save()
                        Aggregate.add_stat(d=str(i), m=devices[i], v='{}-{}-{}-{}'.format(version, date, t, devices[i]), u='US', c='Carrier', c_id='0')

def test_post(client):
    result = client.simulate_post('/api/v1/stats', body=json.dumps(dict(device_hash='1', device_name='cucumber', device_version='14.1-20170101-NIGHTLY-cucumber', device_country='US', device_carrier='Carrier', device_carrier_id='0')))
    assert result.status_code == 200
    assert(Statistic.objects().count() == 1)
    Statistic.objects().delete()
    Aggregate.objects().delete()

def test_get(client):
    create_statistics()
    expected = {
        'model': Aggregate.get_most_popular('model', 90),
        'country': Aggregate.get_most_popular('country', 90),
        'total': Aggregate.get_count(90)
    }
    result = client.simulate_get('/api/v1/stats')
    assert result.status_code == 200
    assert result.json == expected
    Statistic.objects().delete()
    Aggregate.objects().delete()

def test_popular_stats(client):
    create_statistics()
    popular = Aggregate.get_most_popular('model', 90)
    assert(len(popular) == 5)
    Statistic.objects().delete()
    Aggregate.objects().delete()

def test_field_stats(client):
    create_statistics()
    by_field = Aggregate.get_info_by_field("model", "cucumber", "model", "country")
    assert len(by_field.keys()) == 3
    assert by_field['total'] == 1
    print(by_field)
    Statistic.objects().delete()
    Aggregate.objects().delete()

