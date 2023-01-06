package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	_ "github.com/lib/pq"
	_ "github.com/mattn/go-sqlite3"
)

type Config struct {
	DatabaseUri string
}

func (c *Config) Load() {
	c.DatabaseUri = os.Getenv("DATABASE_URI")
}

type Statistic struct {
	Hash      string `json:"device_hash"`
	Name      string `json:"device_name"`
	Version   string `json:"device_version"`
	Country   string `json:"device_country"`
	Carrier   string `json:"device_carrier"`
	CarrierId string `json:"device_carrier_id"`
}

func main() {
	var config Config
	config.Load()

	fmt.Println(config.DatabaseUri)

	db, err := sql.Open("postgres", config.DatabaseUri)
	if err != nil {
		panic(err)
	}
	defer db.Close()

	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(middleware.Timeout(60 * time.Second))
	r.Route("/api/v1/stats", func(r chi.Router) {
		r.Use(middleware.AllowContentEncoding("application/json"))
		r.Post("/", func(w http.ResponseWriter, r *http.Request) {
			stat := Statistic{}
			err := json.NewDecoder(r.Body).Decode(&stat)
			if err != nil {
				// json was invalid, this is a bad request
				w.WriteHeader(http.StatusBadRequest)
				return
			}

			stmt, err := db.Prepare("INSERT INTO statistics(device_id, model, version_raw, country, carrier, carrier_id) VALUES ($1, $2, $3, $4, $5, $6)")
			if err != nil {
				fmt.Printf("error preparing: %s\n", err)
				w.WriteHeader(http.StatusInternalServerError)
				w.Write([]byte("sql error"))
				return
			}
			_, err = stmt.Exec(stat.Hash, stat.Name, stat.Version, stat.Country, stat.Carrier, stat.CarrierId)
			if err != nil {
				fmt.Printf("error execing: %s\n", err)
				w.WriteHeader(http.StatusInternalServerError)
				w.Write([]byte("sql error"))
				return
			}

			w.Write([]byte("neat"))
			w.WriteHeader(200)
		})
	})

	fmt.Println("hi")

	http.ListenAndServe("localhost:8080", r)
}
