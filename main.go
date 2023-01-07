package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	_ "github.com/lib/pq"
	"github.com/lineageos-infra/tribble-tracker/internal/db"
)

type Config struct {
	DatabaseUri string
}

func (c *Config) Load() {
	c.DatabaseUri = os.Getenv("DATABASE_URI")
}

func main() {
	var config Config
	config.Load()

	client, err := db.NewPostgresClient(config.DatabaseUri)
	if err != nil {
		panic(err)
	}
	defer client.Close()

	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(middleware.Timeout(60 * time.Second))
	r.Route("/api/v1/stats", func(r chi.Router) {
		r.Use(middleware.AllowContentEncoding("application/json"))
		r.Post("/", func(w http.ResponseWriter, r *http.Request) {
			stat := db.Statistic{}
			err := json.NewDecoder(r.Body).Decode(&stat)
			if err != nil {
				// json was invalid, this is a bad request
				w.WriteHeader(http.StatusBadRequest)
				return
			}

			err = client.InsertStatistic(stat)
			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				w.Write([]byte("sql error"))
				return
			}

			w.Write([]byte("neat"))
			w.WriteHeader(200)
		})
		r.Get("/", func(w http.ResponseWriter, r *http.Request) {
			type Response struct {
				Model   map[string]int `json:"model"`
				Country map[string]int `json:"country"`
				Version map[string]int `json:"version"`
				Total   map[string]int `json:"total"`
			}
			model, err := client.GetMostPopular("model")
			if err != nil {
			}
			country, err := client.GetMostPopular("country")
			if err != nil {
			}
			version, err := client.GetMostPopular("version")
			if err != nil {
			}
			total, err := client.GetCount()
			if err != nil {
			}

			resp := struct {
				Model   map[string]int `json:"model"`
				Country map[string]int `json:"country"`
				Version map[string]int `json:"version"`
				Total   int            `json:"total"`
			}{
				Model:   model,
				Country: country,
				Version: version,
				Total:   total,
			}

			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(resp)
		})
	})
	r.Get("/robots.txt", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		w.Write([]byte("User-agent: *\nDisallow: /"))
	})

	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		_, err := client.GetMostPopular("version")
		if err != nil {
			fmt.Println(err)
		}
	})

	fmt.Println("hi")

	http.ListenAndServe("localhost:8080", r)
}
