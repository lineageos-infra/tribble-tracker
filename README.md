LineageOS Statistics Backend
=======================
Copyright (c) 2017 The LineageOS Project<br>


Data Collected
=======================

Devices check in (roughly) daily with the following data:

* Device ID: The sha256 of [Settings.Secure.ANDROID_ID](https://developer.android.com/reference/android/provider/Settings.Secure.html#ANDROID_ID). This ID is reset every time the device is wiped.
* Device model, taken from `ro.cm.device` / `ro.lineage.device`.
* Device version, taken from `ro.cm.version` / `ro.lineage.version`. For Lineage builds, this is in the format `VERSION-DATE-TYPE-MODEL`.
* Device country, as reported by the SIM card.
* Device carrier, as reported by the SIM card.
* Device carrier ID, as reported by the SIM card.

Additionally, we record the following:

* Current time the request was made.


Limitations
=======================

* Devices do not retry when a check in fails, it is assumed they will check in the next day. As such, device counts for a given day may be lower than the actual number of devices available.
* Devices lose their ANDROID_ID when they're either wiped. As such, device counts may be high if a large number of wipes have occured during the period (ie, during new release times).
* Devices without sim cards do not report their country, carrier, or carrier ID.

All stats are displayed, you'll want to drop old data if that's a thing you care about. We keep 90 days. Only the last checkin for each device is kept.


Development
=======================

Install [asdf](https://github.com/asdf-vm/asdf)
Run:
```bash
asdf plugin add golang
asdf install
```
