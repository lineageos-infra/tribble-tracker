#[derive(Debug, toasty::Model)]
#[table = "stats"]
pub struct Stat {
    #[key]
    pub device_id: String,

    pub carrier: Option<String>,
    pub carrier_id: Option<String>,
    pub country: Option<String>,
    pub model: Option<String>,
    pub official: Option<bool>,
    pub submit_time: Option<jiff::civil::DateTime>,
    pub version: Option<String>,
    pub version_raw: Option<String>,
}

#[derive(Debug, toasty::Model)]
#[table = "banned"]
pub struct Banned {
    #[key]
    pub model: String,

    #[key]
    pub version: String,

    pub note: Option<String>,
}
