package main

import (
	"encoding/json"
	"fmt"
	"html/template"
	"net"
	"net/http"
	"os"
	"regexp"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/httplog"
	"github.com/lineageos-infra/tribble-tracker/internal/db"
)

var versionRegex = regexp.MustCompile(`^\d\d\.\d`)
var officialRegex = regexp.MustCompile(`\d\d\.\d-\d{8}-NIGHTLY-.*`)

type Config struct {
	DatabasePath string
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
	c.DatabasePath = os.Getenv("DATABASE_PATH")
	if c.DatabasePath == "" {
		c.DatabasePath = "dev.db"
	}
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
	logger := httplog.NewLogger("stats", httplog.Options{LogLevel: "warn", JSON: true})
	var config Config
	config.Load()

	client, err := db.NewSqlite3Client(config.DatabasePath, "rw")
	if err != nil {
		logger.Panic().Err(err).Msg("Failed to get rw database client")
	}
	defer client.Close()

	clientRo, err := db.NewSqlite3Client(config.DatabasePath, "ro")
	if err != nil {
		logger.Panic().Err(err).Msg("Failed to get ro database client")
	}
	defer clientRo.Close()

	// TODO(zif): refresh this on a timer
	banned := db.NewBanned()
	go func() {
		err := banned.Update(clientRo)
		if err != nil {
			logger.Panic().Err(err).Msg("failed to update banned list at startup")
		}
		for range time.Tick(time.Minute * 1) {
			err := banned.Update(clientRo)
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

	rawTmpl, err := os.ReadFile(config.TemplatePath)
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
	r.Use(middleware.RealIP)
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
				return
			}

			if _, found := banned.Models[stat.Name]; found {
				// model is banned, return neat
				w.Write([]byte("neat"))
				w.WriteHeader(200)
				return
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

			// parse version code
			version := versionRegex.FindString(stat.Version)
			if version == "" {
				w.WriteHeader(http.StatusBadRequest)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("version must start with version code (ie, 22.1)"))
				return
			}

			official := officialRegex.MatchString(stat.Version)

			if stat.Country != "Unknown" {
				stat.Country = strings.ToUpper(stat.Country)
			}
			err = client.InsertStatistic(stat, version, official)
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

			model, err := clientRo.GetMostPopular("model", "", "")
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			country, err := clientRo.GetMostPopular("country", "", "")
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			version, err := clientRo.GetMostPopular("version", "", "")
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to read database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to read from database"))
				return
			}
			total, err := clientRo.GetCount("", "")
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

		r.Post("/ban/model", func(w http.ResponseWriter, r *http.Request) {
			host, _, _ := net.SplitHostPort(r.RemoteAddr)
			if host != "127.0.0.1" && host != "::1" {
				w.WriteHeader(http.StatusForbidden)
				return
			}
			data := struct {
				Model string `json:"model"`
				Note  string `json:"note"`
			}{}
			err := json.NewDecoder(r.Body).Decode(&data)
			if err != nil || len(data.Model) == 0 {
				w.WriteHeader(http.StatusInternalServerError)
				return
			}
			err = client.BanModel(data.Model, data.Note)
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to write to the database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to write to the database"))
				return
			}
			w.Header().Set("Content-Type", "text/plain")
			w.Write([]byte("OK"))
		})

		r.Post("/ban/version", func(w http.ResponseWriter, r *http.Request) {
			host, _, _ := net.SplitHostPort(r.RemoteAddr)
			if host != "127.0.0.1" && host != "::1" {
				w.WriteHeader(http.StatusForbidden)
				return
			}
			data := struct {
				Version string `json:"version"`
				Note    string `json:"note"`
			}{}
			err := json.NewDecoder(r.Body).Decode(&data)
			if err != nil || len(data.Version) == 0 {
				w.WriteHeader(http.StatusInternalServerError)
				return
			}
			err = client.BanVersion(data.Version, data.Note)
			if err != nil {
				log := httplog.LogEntry(r.Context())
				log.Error().Err(err).Msg("failed to write to the database")
				w.WriteHeader(http.StatusInternalServerError)
				w.Header().Add("Content-Type", "text/plain")
				w.Write([]byte("failed to write to the database"))
				return
			}
			w.Header().Set("Content-Type", "text/plain")
			w.Write([]byte("OK"))
		})

		r.Get("/ban/list", func(w http.ResponseWriter, r *http.Request) {
			host, _, _ := net.SplitHostPort(r.RemoteAddr)
			if host != "127.0.0.1" && host != "::1" {
				w.WriteHeader(http.StatusForbidden)
				return
			}
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(banned)
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
		left, err := clientRo.GetMostPopular("model", "", "")
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Left = left
		right, err := clientRo.GetMostPopular("country", "", "")
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Right = right
		total, err := clientRo.GetCount("", "")
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Total = total
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
			left, err := clientRo.GetMostPopular("version", thing, name)
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
			left, err := clientRo.GetMostPopular("model", thing, name)
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
			right, err := clientRo.GetMostPopular("carrier", thing, name)
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
			right, err := clientRo.GetMostPopular("country", thing, name)
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

		total, err := clientRo.GetCount(thing, name)
		if err != nil {
			log := httplog.LogEntry(r.Context())
			log.Error().Err(err).Msg("failed to read database")
			w.WriteHeader(http.StatusInternalServerError)
			w.Header().Add("Content-Type", "text/plain")
			w.Write([]byte("failed to read from database"))
			return
		}
		data.Total = total
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
