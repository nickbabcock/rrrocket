# rrrocket

![ci](https://github.com/nickbabcock/rrrocket/workflows/ci/badge.svg)

rrrocket parses a Rocket League replay file and outputs JSON.

Underneath rrrocket is the general rocket league parsing library: [boxcars](https://crates.io/crates/boxcars)

## Installation

Download the appropriate bundle from the [releases page](https://github.com/nickbabcock/rrrocket/releases/latest):

- For Windows, you'll want the one labeled `windows-msvc`
- For Linux, you'll want the one labeled `linux-musl`
- For macOS, you'll want the only one labeled `apple`

## Usage

```
Parses Rocket League replay files and outputs JSON with decoded information

USAGE:
    rrrocket [FLAGS] [input]...

FLAGS:
    -n, --network-parse    parses the network data of a replay instead of skipping it
    -c, --crc-check        forces a crc check for corruption even when replay was successfully parsed
        --dry-run          parses but does not write JSON output
    -h, --help             Prints help information
    -j, --json-lines       output multiple files to stdout via json lines
    -m, --multiple         parse multiple replays in provided directories. Defaults to writing to a sibling JSON file,
                           but can output to stdout with --json-lines
    -p, --pretty           output replay as pretty-printed JSON
    -V, --version          Prints version information

ARGS:
    <input>...    Rocket League replay files
```

## Output

A sample output of the JSON from rrrocket:

```json
{
  "header_size": 4768,
  "header_crc": 337843175,
  "major_version": 868,
  "minor_version": 12,
  "game_type": "TAGame.Replay_Soccar_TA",
  "properties": {
    "TeamSize": 3,
    "Team0Score": 5,
    "Team1Score": 2,
    "Goals": [
      {
        "PlayerName": "Cakeboss",
        "PlayerTeam": 1,
        "frame": 441
      },
      // all the goals
    ]
    // and many more properties
  }
```

If network parsed is enabled then an attribute (snipped) looks something like:

```json
{
  "actor_id": 6,
  "stream_id": 51,
  "attribute": {
    "RigidBody": {
      "sleeping": true,
      "location": {
        "bias": 16384,
        "dx": 16384,
        "dy": 16384,
        "dz": 25658
      },
      "x": 1,
      "y": 1,
      "z": 1,
      "linear_velocity": null,
      "angular_velocity": null
    }
  }
}
```

## Queries with jq

Since rrrocket outputs json, [jq](https://stedolan.github.io/jq/) is a natural query tool. Here are some questions that rrrocket and jq can answer together.

Want to find out which non 3v3 games had a score difference greater than 2?

```
rrrocket --json-lines --multiple ~/projects/boxcars/assets/replays/good/ \
  | jq -c 'if (.replay.properties.TeamSize != 3 and
      (((.replay.properties.Team0Score // 0) - (.replay.properties.Team1Score // 0) | length) > 2)) then .file else empty end'
```

Top combined score?

```
rrrocket --json-lines --multiple ~/projects/boxcars/assets/replays/good/ \
  | jq -c  '{(.file): (.replay.properties.Team0Score // 0 + .replay.properties.Team1Score // 0)}' \
  | sort -n -k2 -t ':'
```

Games with certain attributes?

```
rrrocket --json-lines --multiple ~/projects/boxcars/assets/replays/good/ \
  | jq -c 'if (.replay.objects | contains(["Archetypes.Ball.Ball_Breakout"])) then .file else empty end'
```

## boxcapy

boxcapy is a python script that ingests given JSON files that have been created
by `rrrocket` (and only needs header information). The command below is the one
I use to generate the JSON files:

```bash
find . -type f -iname "*.replay" | xargs ~/rrrocket -m 
```

To have your graphs saved into your directory follow the below instructions:

- Since the graphs are in the style of XKCD, one has to install the Humor Sans font before continuing (eg. `apt install fonts-humor-sans`)
- Run on generated JSON files: `./boxcapy/rocket-plot.py *.json --headless`
