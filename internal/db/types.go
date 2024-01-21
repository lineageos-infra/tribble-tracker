package db

type Statistic struct {
	Hash      string `json:"device_hash"`
	Name      string `json:"device_name"`
	Version   string `json:"device_version"`
	Country   string `json:"device_country"`
	Carrier   string `json:"device_carrier"`
	CarrierId string `json:"device_carrier_id"`
}

func (*stat Statistic) Normalize() {
    if len(country) == 2 {
        stat.Country = strings.ToUpper(stat.Country)
    }
}

type Stat struct {
	Item  string
	Count int
}

type Banned struct {
	Versions map[string]bool
	Models   map[string]bool
}

func NewBanned() *Banned {
	banned := Banned{}
	banned.Models = make(map[string]bool)
	banned.Versions = make(map[string]bool)
	return &banned
}
