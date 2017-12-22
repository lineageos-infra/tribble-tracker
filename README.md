LineageOS Statistics Backend
=======================
Copyright (c) 2017 The LineageOS Project<br>


Data Collected
=======================

Devices check in (roughly) daily with the following data:

* Device ID: The sha256 of [Settings.Secure.ANDROID_ID](https://developer.android.com/reference/android/provider/Settings.Secure.html#ANDROID_ID). This ID is reset every time the device is wiped.
* Device model, taken from `ro.cm.device`.
* Device version, taken from `ro.cm.version`. For Lineage builds, this is in the format `VERSION-DATE-TYPE-MODEL`.
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

All stats displayed are an aggregate of the last 90 days. The magnitude of any given device is assumed to be accurate (ie, 1% of all devices are a specific model).

