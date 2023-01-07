package main

import (
	"encoding/json"
	"fmt"
	"html/template"
	"io/ioutil"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	_ "github.com/lib/pq"
	"github.com/lineageos-infra/tribble-tracker/internal/db"
)

type Config struct {
	DatabaseUri  string
	TemplatePath string
	StaticPath   string
}

type TemplateData struct {
	Left      []db.Stat
	Right     []db.Stat
	LeftName  string
	RightName string
	Thing     string
	Total     int
}

func (c *Config) Load() {
	c.DatabaseUri = os.Getenv("DATABASE_URI")
	c.TemplatePath = os.Getenv("TEMPLATE_PATH")
	if c.TemplatePath == "" {
		c.TemplatePath = "templates/index.html"
	}
	c.StaticPath = os.Getenv("STATIC_PATH")
	if c.StaticPath == "" {
		c.StaticPath = "static/"
	}
}

func main() {
	var config Config
	config.Load()

	client, err := db.NewPostgresClient(config.DatabaseUri)
	if err != nil {
		panic(err)
	}
	defer client.Close()

	rawTmpl, err := ioutil.ReadFile(config.TemplatePath)
	if err != nil {
		panic(err)
	}
	funcMap := template.FuncMap{
		"ToLower": strings.ToLower,
	}
	tmpl, err := template.New("stats").Funcs(funcMap).Parse(string(rawTmpl))
	if err != nil {
		panic(err)
	}

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
			resp := struct {
				Model   map[string]int `json:"model"`
				Country map[string]int `json:"country"`
				Version map[string]int `json:"version"`
				Total   int            `json:"total"`
			}{
				Model:   map[string]int{},
				Country: map[string]int{},
				Version: map[string]int{},
				Total:   0,
			}

			model, err := client.GetMostPopular("model")
			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				fmt.Println(err)
			}
			country, err := client.GetMostPopular("country")
			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				fmt.Println(err)
			}
			version, err := client.GetMostPopular("version")
			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				fmt.Println(err)
			}
			total, err := client.GetCount()
			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				fmt.Println(err)
			}

			for _, i := range model {
				resp.Model[i.Item] = i.Count
			}
			for _, i := range country {
				resp.Country[i.Item] = i.Count
			}
			for _, i := range version {
				resp.Version[i.Item] = i.Count
			}
			resp.Total = total

			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(resp)
		})
	})
	r.Get("/robots.txt", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		w.Write([]byte("User-agent: *\nDisallow: /"))
	})

	staticServer := http.FileServer(http.Dir(config.StaticPath))
	r.Handle("/static/*", http.StripPrefix("/static", staticServer))

	r.Get("/", func(w http.ResponseWriter, r *http.Request) {

		data := TemplateData{
			LeftName:  "Model",
			RightName: "Country",
			Thing:     "active",
		}
		left, err := client.GetMostPopular("model")
		if err != nil {
			fmt.Println(err)
		}
		data.Left = left
		right, err := client.GetMostPopular("country")
		if err != nil {
			fmt.Println(err)
		}
		data.Right = right
		total, err := client.GetCount()
		if err != nil {
			fmt.Println(err)
		}
		data.Total = total
		tmpl.Execute(w, data)
	})

	http.ListenAndServe(":8080", r)
}
