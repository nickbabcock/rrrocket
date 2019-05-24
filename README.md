# rrrocket

[![Build Status](https://travis-ci.org/nickbabcock/rrrocket.svg?branch=master)](https://travis-ci.org/nickbabcock/rrrocket) [![Build status](https://ci.appveyor.com/api/projects/status/939bi13urfp8w1n6?svg=true)](https://ci.appveyor.com/project/nickbabcock/rrrocket)

rrrocket parses a Rocket League replay file and outputs JSON.

Underneath rrrocket is the general rocket league parsing library: [boxcars](https://crates.io/crates/boxcars)

## Installation

Download the appropriate bundle from the [releases page](https://github.com/nickbabcock/rrrocket/releases/latest):

- For Windows, you'll want the one labeled `windows-msvc`
- For Linux, you'll want the one labeled `linux-musl`
- For macOS, you'll want the only one labeled `apple`

## Usage

```
USAGE:
    rrrocket [FLAGS] [input]...

FLAGS:
    -n, --network-parse    parses the network data of a replay instead of skipping it
    -c, --crc-check        forces a crc check for corruption even when replay was successfully parsed
        --dry-run          parses but does not write JSON output
    -h, --help             Prints help information
    -m, --multiple         parse multiple replays, instead of writing JSON to stdout, write to a sibling JSON file
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

## boxcapy

boxcapy is a python script that ingests given JSON files that have been created
by `rrrocket` (and only needs header information). The command below is the one
I use to generate the JSON files:

```bash
find . -type f -iname "*.replay" | xargs ~/rrrocket -m 
```

To have your graphs saved into your directory follow the below instructions:

- Since the graphs are in the style of XKCD, one has to install the Humor Sans font before continuing (eg. `apt install fonts-humor-sans`)

### Python 2

- Install [pipenv](https://docs.pipenv.org/install.html#installing-pipenv)
- Install dependencies `pipenv --two && pipenv install`
- Run on generated JSON files: `pipenv run boxcapy/rocket-plot.py ~/Demos/*.json --headless`

### Python 3

- Install [pipenv](https://docs.pipenv.org/install.html#installing-pipenv)
- Install dependencies `pipenv --three && pipenv install --skip-lock`
- Run on generated JSON files: `pipenv run boxcapy/rocket-plot.py ~/Demos/*.json --headless`