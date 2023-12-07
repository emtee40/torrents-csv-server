# Torrents.csv

<!-- Torrents.csv - An open source, collaborative repository of torrents, with a self-hostable web server.   -->

[Demo Server](https://torrents-csv.com)

`Torrents.csv` is a _collaborative_ repository of torrents, consisting of a searchable `torrents.csv` file. It aims to be a universal file system for popular data.

Its initially populated with a January 2017 backup of the pirate bay, and new torrents are periodically added from various torrents sites. It comes with a self-hostable [Torrents.csv webserver](https://torrents-csv.com), a command line search, and a folder scanner to add torrents.

`Torrents.csv` will only store torrents with at least one seeder to keep the file small, will be periodically purged of non-seeded torrents, and sorted by infohash.

![img](https://i.imgur.com/yTFuwpv.png)

To request more torrents, or add your own, go [here](https://git.torrents-csv.com/heretic/torrents-csv-data).

Made with [Rust](https://www.rust-lang.org), [ripgrep](https://github.com/BurntSushi/ripgrep), [Actix](https://actix.rs/), [Perseus](https://framesurge.sh/perseus/en-US/), and [Sycamore](https://sycamore-rs.netlify.app/).

## Webserver

`Torrents.csv` comes with a simple webserver. [Demo Server](https://torrents-csv.com)

### Docker

```
wget https://git.torrents-csv.com/heretic/torrents-csv-server/raw/branch/main/docker/docker-compose.yml
wget https://git.torrents-csv.com/heretic/torrents-csv-server/raw/branch/main/docker/nginx.conf
docker-compose up -d
```

And goto http://localhost:8904

### Docker Development

```
git clone --recurse-submodules https://git.torrents-csv.com/heretic/torrents-csv-server
cd torrents-csv-server/docker/dev
./docker_update.sh
# For the front end, check out http://git.torrents-csv.com/heretic/torrents-csv-ui-perseus
```

## Command Line Searching

### Requirements

- [ripgrep](https://github.com/BurntSushi/ripgrep)

### Running

```
git clone --recurse-submodules https://git.torrents-csv.com/heretic/torrents-csv-server
cd torrents-csv-server
./search.sh "bleh season 1"
bleh season 1 (1993-)
	seeders: 33
	size: 13GiB
	link: magnet:?xt=urn:btih:INFO_HASH_HERE
```

## API

A JSON output of search results is available at:

`http://localhost:8904/service/search?q=[QUERY]&size=[NUMBER_OF_RESULTS]&page=[PAGE]`

New torrents are at:

`http://localhost:8904/service/new?size=[NUMBER_OF_RESULTS]&page=[PAGE]`
