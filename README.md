# sulu-cli
Convert OSM protobuf files to routable networks

# Description
Sulu processes `osm.pbf` files of Openstreetmap data. 
These files contain data conforming to the [Openstreetmap data model](https://wiki.openstreetmap.org/wiki/Elements). 
To create routable networks we are primarly concerned with [ways](https://wiki.openstreetmap.org/wiki/Way), but only
those with select [tags](https://wiki.openstreetmap.org/wiki/Tags).

Sulu uses a json file to determine the collection of tags a way must have in order to be included in the network.

In addition to selecting these appropriate ways, sulu also creates a network topology - breaking ways into edges 
at nodes that are shared by two (or more) selected ways.

The graph can be output using the (vector) spatial formats supported by GDAL using [rust-gdal](https://github.com/georust/gdal),
or geojson using [georust's implementation of geojson](https://github.com/georust/geojson).

# Installation
An installation of gdal (version >= 2.2) is required. 

* On ubuntu (>= 18.04 LTS) use
```
sudo apt install gdal
```
* Mac OS
```
brew install gdal
```

Sulu can be built from source using rust. 
Relevant rust tooling can be installed using [rustup](https://rustup.rs/). 
Once rust and cargo are installed, use the following to install sulu

```
git clone https://github.com/kinesisptyltd/sulu-cli.git
cd sulu-cli
cargo install .
```

# Usage
```
>>> sulu --help
Sulu 0.1.0
Tom Watson <tom.watson@kinesis.org>
Converts osm.pbf files into routable networks

USAGE:
    sulu [OPTIONS] <INPUT> <OUTPUT> <GRAPH-CONFIG>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <format>        How sulu will write the output file [default: geojson]  [possible values: geojson, gdal]
    -d <driver>        Short name of gdal driver to use to write the output. See
                       https://gdal.org/drivers/vector/index.html [default: gpkg]

ARGS:
    <INPUT>           The osm.pbf file to process
    <OUTPUT>          The output file
    <GRAPH-CONFIG>    File containing the definition of the graph
```

For example:
```
sulu input.osm.pbf output.gpkg config.json -f gdal -d gpkg
```

## The graph config file
The graph config file is a json file that determines what ways are selected to be included in the graph.
An example can be seen in [the examples folder](https://raw.githubusercontent.com/kinesisptyltd/sulu-cli/main/examples/basic_roads.json?token=ACNOJ7TRASMQ23LABFHS2FK7LK7YI).

A brief description of the config file is included below. A way will be selected if it matches the Matchers in `requires` and not the Matchers in `excludes`.
If the way would match multiple Graph Config Options it is assigned the first option that it matches.

These config files can be used to describe particular networks - driving, walking, cycling, rail etc - and the options can provide distinction between
the various links included in the network (dedicated cycleway vs on-road cycle, fast roads vs slow roads etc).
```
Graph Config
============
- name: The name for this configuration
- options: An ordered list of Graph Config Options a way might satisfy

Graph Config Option
===================
- name: The name for this option
- requires: An ordered list of Matchers the way _must_ satisfy
- excludes: An ordered list of Matchers the way _must not_ satisfy

Matcher
=======
- key: The key of the tag
- kind: The kind of match - one of "in-list", "exact" and "all"

Kind
====
- in-list: An array of strings. 
  - This matcher is matched if the tag value is in the list
- exact: a string. 
  - This matcher is matched only if the tag value is this value
- all 
  - This matcher is matched for any tag value
```

# Tips and tricks
 * You can query features on [openstreetmap](https://www.openstreetmap.org) to see what tags they have
 * The OSM wiki is a good resource to see how things are (or should be) tagged [for example, the key=highway tag](https://wiki.openstreetmap.org/wiki/Key:highway)
 * Sulu needs to check every feature in the `osm.pbf` file to see if it matches your config, so using another osm tool to remove features that
 are definitely superfluous can be helpful. For example, snipping out a bounding box using `osmconvert` (`osmconvert -b=left,bottom,right,top -o=output.osm.pbf input.osm.pbf`).
 * Use graph config options to partition ways into things that are useful for you. E.g. matching motorways separately to residential streets and applying different
 speed-limit or capacity assumptions when doing traffic modelling.

# Useful links
 * [OSM description of highways](https://wiki.openstreetmap.org/wiki/Highways)
 * [OSM convert for pre-processing `osm.pbf` files](https://wiki.openstreetmap.org/wiki/Osmconvert)
 * [Geofabrik data extracts for `osm.pbf` files for large areas](https://download.geofabrik.de/)
