from __future__ import unicode_literals

import pytest

from falcon import testing
from app import app
import json


@pytest.fixture()
def client():
    return testing.TestClient(app)


def test_post(client):
    result = client.simulate_post(
        "/api/v1/stats",
        body=json.dumps(
            dict(
                device_hash="1",
                device_name="cucumber",
                device_version="14.1-20170101-NIGHTLY-cucumber",
                device_country="US",
                device_carrier="Carrier",
                device_carrier_id="0",
            )
        ),
    )
    assert result.status_code == 200


def test_api(client):
    result = client.simulate_get("/api/v1/stats")
    assert result.status_code == 200


def test_index(client):
    result = client.simulate_get("/")
    assert result.status_code == 200
