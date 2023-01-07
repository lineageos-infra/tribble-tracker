package db

import (
	"database/sql"
	"fmt"
	"time"

	_ "github.com/lib/pq"
)

type postgresClient struct {
	db *sql.DB
}

func NewPostgresClient(DatabaseUri string) (*postgresClient, error) {
	db, err := sql.Open("postgres", DatabaseUri)
	if err != nil {
		return nil, err
	}

	client := &postgresClient{db: db}
	return client, nil

}

func (c *postgresClient) Close() {
	c.db.Close()
}

func (c *postgresClient) InsertStatistic(stat Statistic) error {
	stmt, err := c.db.Prepare(`
	INSERT INTO stats(device_id, model, version_raw, country, carrier, carrier_id) 
		VALUES ($1, $2, $3, $4, $5, $6)
		ON CONFLICT (device_id) DO UPDATE
		SET model=$2, version_raw=$3, country=$4, carrier = $5, carrier_id = $6, submit_time=now();
	`)

	if err != nil {
		return err
	}

	_, err = stmt.Exec(stat.Hash, stat.Name, stat.Version, stat.Country, stat.Carrier, stat.CarrierId)
	if err != nil {
		return err
	}

	return nil
}

func (c *postgresClient) GetMostPopular(field string, column string, filterable string) ([]Stat, error) {
	whitelist := map[string]interface{}{
		"version": nil,
		"model":   nil,
		"country": nil,
		"carrier": nil,
	}
	if _, ok := whitelist[field]; !ok {
		// field wasn't valid, reject
		return nil, fmt.Errorf("invalid field: %s", field)
	}
	var rows *sql.Rows
	if column == "" {
		stmt, err := c.db.Prepare(fmt.Sprintf(`
			SELECT %s, count(%s) FROM stats
			GROUP BY %s
			ORDER BY count DESC
			`, field, field, field))
		if err != nil {
			return nil, err
		}

		rows, err = stmt.Query()
		if err != nil {
			return nil, err
		}
	} else {
		if _, ok := whitelist[column]; !ok {
			// column wasn't valid, reject
			return nil, fmt.Errorf("invalid column: %s", field)
		}

		stmt, err := c.db.Prepare(fmt.Sprintf(`
			SELECT %s, count(%s) FROM stats
			WHERE %s = $1
			GROUP BY %s
			ORDER BY count DESC
		`, field, field, column, field))
		if err != nil {
			return nil, err
		}
		rows, err = stmt.Query(filterable)
		if err != nil {
			return nil, err
		}
	}
	defer rows.Close()
	var result []Stat
	for rows.Next() {
		var name string
		var count int
		err := rows.Scan(&name, &count)
		if err != nil {
		}
		result = append(result, Stat{Item: name, Count: count})

	}
	err := rows.Err()
	if err != nil {
		return nil, err
	}
	return result, nil
}
func (c *postgresClient) GetCount() (int, error) {
	row := c.db.QueryRow(`SELECT count(device_id) FROM stats`)
	err := row.Err()
	if err != nil {
		return 0, err
	}
	var count int
	row.Scan(&count)
	return count, nil

}
func (c *postgresClient) DropOld() error {
	stmt, err := c.db.Prepare(`DROP FROM stats WHERE submit < $1`)
	if err != nil {
		return err
	}
	_, err = stmt.Exec(stmt, time.Now().Add(-90*24*time.Hour))
	return err

}
