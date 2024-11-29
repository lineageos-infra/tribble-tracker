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
	"github.com/go-chi/httplog"
	_ "github.com/lib/pq"
	"github.com/lineageos-infra/tribble-tracker/internal/db"
	"golang.org/x/text/language"
	"golang.org/x/text/message"
	"golang.org/x/text/number"
)

type Config struct {
	DatabaseUrl  string
	TemplatePath string
	StaticPath   string
}

type TemplateData struct {
	Left      []db.Stat
	Right     []db.Stat
	LeftName  string
	RightName string
	Thing     string
	Total     string
}

func (c *Config) Load() {
	c.DatabaseUrl = os.Getenv("DATABASE_URL")
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
	logger := httplog.NewLogger("stats", httplog.Options{JSON: true})
	printer := message.NewPrinter(language.English)
	var config Config
	config.Load()

	client, err := db.NewPostgresClient(config.DatabaseUrl)
	if err != nil {
		logger.Panic().Err(err).Msg("Failed to get database client")
	}
	defer client.Close()

	// TODO(zif): refresh this on a timer
	banned := db.NewBanned()
	go func() {
		err := banned.Update(client)
		if err != nil {
			logger.Panic().Err(err).Msg("failed to update banned list at startup")
		}
		for range time.Tick(time.Minute * 1) {
			err := banned.Update(client)
			if err != nil {
				logger.Error().Err(err).Msg("failed to update banned list")
			}
		}
	}()

	// purge older than 90d on a timer
	go func() {
		err := client.DropOld()
		if err != nil {
			logger.Panic().Err(err).Msg("Failed to drop old stats at startup")
		}
		for range time.Tick(time.Hour * 6) {
			err := client.DropOld()
			if err != nil {
				logger.Warn().Err(err).Msg("Failed to drop old stats")
			}
		}
	}()

	rawTmpl, err := ioutil.ReadFile(config.TemplatePath)
	if err != nil {
		logger.Panic().Err(err).Msg("Failed to load template from file")
	}
	funcMap := template.FuncMap{
		"ToLower": strings.ToLower,
		"mod":     func(i int, j int) bool { return i%j == 0 },
		"inc":     func(i int) int { return i + 1 },
	}
	tmpl, err := template.New("stats").Funcs(funcMap).Parse(string(rawTmpl))
	if err != nil {
		logger.Panic().Err(err).Msg("Failed to parse template")
	}

	r := chi.NewRouter()
	r.Use(middleware.Timeout(60 * time.Second))
	r.Use(httplog.RequestLogger(logger))
	r.Route("/api/v1/stats", func(r chi.Router) {
		r.Use(middleware.AllowContentEncoding("application/json"))
		r.Post("/", func(w http.ResponseWriter, r *http.Request) {
			defer r.Body.Close()
			stat := db.Statistic{}
			err := json.NewDecoder(r.Body).Decode(&stat)
			if err != nil {
				// json was invalid, this is a bad request
				w.WriteHeader(http.StatusBadRequest)
				w.Header().Add("Content-type", "text/plain")
				w.Write([]byte("failed to decode json, was the format wrong?"))
				return
			}

			if _, found := banned.Versions[stat.Version]; found {
				// version is banned, return neat
				w.Write([]byte("neat"))
				w.WriteHeader(200)
			}

			if _, found := banned.Models[stat.Name]; found {
				// model is banned, return neat
				w.Write([]byte("neat"))
				w.WriteHeader(200)
			}

			// version _must_ contain  model
			if stat.Name != "x86_64" && !strings.HasSuffix(stat.Version, stat.Name) {
				w.WriteHeader(http.StatusBadRequest)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("version string must end with -model"))
				return
			}

			if len(stat.Country) != 2 && stat.Country != "Unknown" {
				w.WriteHeader(http.StatusBadRequest)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("country must be a two letter iso code"))
				return
			}

			if stat.Country != "Unknown" {
				stat.Country = strings.ToUpper(stat.Country)
			}
			err = client.InsertStatistic(stat)
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to insert statistic")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
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

			model, err := client.GetMostPopular("model", "", "")
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			country, err := client.GetMostPopular("country", "", "")
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			version, err := client.GetMostPopular("version", "", "")
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			total, err := client.GetCount("", "")
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
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
		left, err := client.GetMostPopular("model", "", "")
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Left = left
		right, err := client.GetMostPopular("country", "", "")
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Right = right
		total, err := client.GetCount("", "")
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Total = printer.Sprintf("%v", number.Decimal(total, number.Precision(3)))
		err = tmpl.Execute(w, data)
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
	})

	r.Get("/{thing}/{name}", func(w http.ResponseWriter, r *http.Request) {
		thing := chi.URLParam(r, "thing")
		name := chi.URLParam(r, "name")

		data := TemplateData{
			Thing: name,
		}
		// do Left
		if thing == "model" {
			data.LeftName = "Version"
			left, err := client.GetMostPopular("version", thing, name)
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			data.Left = left
		} else {
			data.LeftName = "Model"
			left, err := client.GetMostPopular("model", thing, name)
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			data.Left = left
		}
		if thing == "country" {
			data.RightName = "Carrier"
			right, err := client.GetMostPopular("carrier", thing, name)
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			data.Right = right
		} else {
			data.RightName = "Country"
			right, err := client.GetMostPopular("country", thing, name)
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			data.Right = right
		}

		total, err := client.GetCount(thing, name)
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Total = printer.Sprintf("%v", number.Decimal(total, number.Precision(3)))
		switch thing {
		case "model":
			data.Thing = name
		case "country":
			if name == "Unknown" {
				data.Thing = "Unknown country"
			} else {
				data.Thing = name
			}
		case "version":
			data.Thing = fmt.Sprintf("version %s", name)
		case "carrier":
			if name == "Unknown" {
				data.Thing = "Unknown country"
			} else {
				data.Thing = name
			}
		}
		err = tmpl.Execute(w, data)
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to render template")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to render template"))
			return
		}
	})

	http.ListenAndServe(":8080", r)
}
