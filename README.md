# rrrocket

[![Build Status](https://travis-ci.org/nickbabcock/rrrocket.svg?branch=master)](https://travis-ci.org/nickbabcock/rrrocket) [![Build status](https://ci.appveyor.com/api/projects/status/939bi13urfp8w1n6?svg=true)](https://ci.appveyor.com/project/nickbabcock/rrrocket)

rrrocket is what a cli utilizing the [boxcars
library](https://crates.io/crates/boxcars).  rrrocket parses a Rocket League
replay file and outputs JSON. The executable has been built for many platforms,
so head on over to the [latest
release](https://github.com/nickbabcock/rrrocket/releases/latest) and download
the appropriate bundle. If you're not sure which bundle to download, here are
the most likely options:

- For Windows, you'll want the one labeled `windows-msvc`
- For Linux, you'll want the one labeled `linux-musl`
- For macOS, you'll want the only one labeled `apple`

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

# boxcapy

boxcapy is a python script that ingests given JSON files that have been created
by `rrrocket`. The command below is the one I use to generate the JSON files:

```bash
find . -type f -iname "*.replay" | xargs ~/rrrocket -m 
```

To have your graphs saved into your directory follow the below instructions:

- Since the graphs are in the style of XKCD, one has to install the Humor Sans font before continuing (eg. `apt install fonts-humor-sans`)

## Python 2

- Install [pipenv](https://docs.pipenv.org/install.html#installing-pipenv)
- Install dependencies `pipenv --two && pipenv install`
- Run on generated JSON files: `pipenv run boxcapy/rocket-plot.py ~/Demos/*.json --headless`

## Python 3

- Install [pipenv](https://docs.pipenv.org/install.html#installing-pipenv)
- Install dependencies `pipenv --three && pipenv install --skip-lock`
- Run on generated JSON files: `pipenv run boxcapy/rocket-plot.py ~/Demos/*.json --headless`
