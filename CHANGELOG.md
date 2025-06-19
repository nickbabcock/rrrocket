# v0.10.5 - June 19th, 2025

- Support for v2.53 replays

# v0.10.4 - March 17th, 2025

- Improved support for v2.49 replays

# v0.10.3 - March 16th, 2025

- Initial support for v2.49 replays

# v0.10.2 - February 1st, 2025

- Huge performance win for musl builds when processing files in parallel
- Performance win with a lighterweight zip file implementation 

# v0.10.1 - January 20th, 2025

- Improve split shot support

# v0.10.0 - January 17th, 2025

Some minor breaking changes in JSON output

- Update `NewColor` product value from i32 to u32
- Update logo data attribute field structure
- Improve support for knockout replays
- Improve support for tutorial replays

# v0.9.16 - January 2nd, 2025

- Support rocket host replays
- Fix edge case of failing to parse saves with UTF-16 player names
- Fix regression on Replays with vote actors
- Performance improvements

# v0.9.15 - December 13th, 2024

- Improved support for Freeplay replays
- Fix double jump decoded as dodge
- Improved replay zip performance and reduced perceived memory usage

# v0.9.14 - December 10th, 2024

- Support for v2.46 replays (this may not be complete, so additional bug reports are welcome).

# v0.9.13 - October 25th, 2024

- Preliminary v2.45 support for struct header properties and empty strings

# v0.9.12 - September 9th, 2024

- Support RL 2.43 replays
- Stable JSON output
- (Experimental) support for processing replays from a zip file

# v0.9.11 - April 21st, 2024

- Support RL 2.39 replays

# v0.9.10 - November 4th, 2023

- Add support for bot replays

# v0.9.9 - June 16th, 2023

- Add support for post match celebration attribute

# v0.9.8 - April 15th, 2023

- Support knockout replays

# v0.9.7 - December 3rd, 2022

- Support parsing patch v2.23 replays

# v0.9.6 - November 7th, 2022

- Improve dropshot compatibility

# v0.9.5 - October 15th, 2022

- Support for parsing patch v2.21 replay attributes

# v0.9.4 - May 10th, 2022

- Support parsing voice chat related attributes (patch v2.15)

# v0.9.3 - November 28th, 2021

- Support parsing impulse attributes (patch v2.08)

## v0.9.2 - August 22nd 2021

- Support for recent rumble replays

## v0.9.1 - August 14th 2021

- Support for rocket league v2.01 replays

## v0.9.0 - July 30th 2021

The byte header property has been updated from looking like:

```json
{
  "Platform": 0
}
```

to

```json
{
  "Platform": {
    "kind": "OnlinePlatform",
    "value": "OnlinePlatform_Steam"
  }
}
```

## v0.8.9 - February 7th 2021

* Support for gridiron replays

## v0.8.8 - January 8th 2021

* Support for replays that should contain a trailer but lack one

## v0.8.7 - December 25th 2020

* Support for additional RLCS / LAN replays

## v0.8.6 - December 9th 2020

* Support rumble pickups from latest replays

## v0.8.5 - December 9th 2020

* nothing to see here

## v0.8.4 - October 5th 2020

* Support additional attributes from latest replays

## v0.8.3 - September 28th 2020

* Support latest rocket league patch (1.82) for Epic IDs

## v0.8.2 - June 17th 2020

* Support latest rocket league patch (1.78) for demolish fx attributes

## v0.8.1 - April 20th 2020

* Support latest rocket league patch (1.76) for heatseeker matches.

## v0.8.0 - April 2nd 2020

* Update replay parser to latest version, which switches some network attribute fields from unsigned to signed and names the fields more appropriately.

## v0.7.2 - March 13th 2020

* Update replay parser to latest version to be able to parse replays from the latest 1.74 patch

## v0.7.1 - March 7th 2020

* Update replay parser to latest version for about a 20% bump in performance

## v0.7.0 - February 24th 2020

* Update replay parser to latest version. The biggest change will cause the
  output of RigidBody rotations to be different (ie accurate). Check [boxcar's release
  notes](https://github.com/nickbabcock/boxcars/blob/master/CHANGELOG.md#v070---february-21st-2020)
  for more information.

## v0.6.2 - December 19th 2019

* Update replay parser to be more resiliant against crafted inputs
* Exit gracefully when output is piped to `head` (broken pipes)

## v0.6.1 - November 19th 2019

* Update replay parser to latest version:
  * Improved error messages
  * Support decoding replays that contain duplicate object ids
  * Support decoding RLCS / Lan replays
  * Support decoding replays with many actors

## v0.6.0 - November 7th 2019

- Recursively scan given directories when the `--multiple` flag is given
- Lazily and iteratively scan the given directories when `--multiple` flag is given. Previous behavior would keep a buffer of all the files found
- Mmap found files in case one has a 10GB iso file named `my-iso.replay`
- Iteratively print json lines when `--multiple` and `--json-lines` are used.
- Update to boxcars 0.6 for latest support of attributes
- `--dry-run` with `--multiple` has been changed to print success when a file parses successfully and the error message of each file that failed to parse. It will now always return 0 in event of replay parsing failures.

## v0.5.0 - October 21th 2019

Previously if one wanted replay data sent to stdout, they could only parse a single file at a time. This is no longer the case with the combination of `--multiple` and `--json-lines`. When a directory and / or multiple replays are provided on the command line, each replay will print its content on a single line. This format is called [json lines](http://jsonlines.org/). `--pretty` is ineffective when `--multiple --json-lines` is used.

## v0.4.4 - October 20th 2019

* Update replay parser for latest replays:
  * v1.68 replays
  * Parse replays with QQ ids
  * Parse replays from the tutorial
  * Parse replays with the anniversary ball
  * Parse replays that contain a ps4 platform id in the header

## v0.4.3 - September 9th 2019

* Update replay parser to latest version (boxcars v0.5.0), which brings a nice performance boost when asked to calculate crc or checking against a corrupted replay

## v0.4.2 - September 4th 2019

* Update replay parser to latest version:
  * Support for patch v1.66 games
  * Include attribute object id on actor update, so now one can more easily derive the attribute's name with `replay.objects[attribute.object_id]`

## v0.4.1 - August 13th 2019

* Update replay parser to latest version:
  * Support for haunted and rugby games.
  * Improvement to error handling that gives detailed error messages on what new / updated actor may have received changes in the RL update. These error messages should only be helpful debugging new updates.
  * Several security fixes:
    * Malicious user could craft NumFrames property to be obscenely high and run the machine out of memory. An error is now thrown if the requested number of frames is greater than the number of bytes remaining.
    * A class's network cache that referenced an out of range object id would cause a index out of bound panic. Now an error is raised.
    * Other fixes are for panics in debug builds

## v0.4.0 - June 11th 2019

* 4x performance improvement when printing json to stdout
* Accept replays piped via stdin

## v0.3.0 - June 10th 2019

* Add `-p/--pretty` flag for pretty printing the JSON output

## v0.2.13 - June 5th 2019

* Update replay parser to be compatible with v1.63

## v0.2.12 - June 2nd 2019

* CRC content changed from signed 32bits to unsigned
* Expose additional information about remote ids on reservations

## v0.2.11 - May 24th 2019

* Update replay parser to be compatible with more replays

## v0.2.10 - April 25th 2019

* Serialize 64bit numbers as strings, so that JSON parsers don't lose any data
  in parsing them as 64bit floating point
  * Javascript numbers are 64bit floating point. 64bit integers can't be
    represented wholly in floating point notation. Thus serialize them as
    strings so that downstream applications can decide on how best to interpret
    large numbers (like 76561198122624102). Affects Int64, QWord, Steam, and
    XBox attributes.
* QWord header property changes from i64 to u64 as some pointed out that
  negative numbers didn't make sense for QWord properties (OnlineId)

## v0.2.9 - April 22nd 2019

* Update replay parser to be compatible with v1.61

## v0.2.8 - April 5th, 2019

* Release for rrrocket's new home (split from the boxcars repo). No changes.

## v0.2.7 - April 4th, 2019

* Update replay parser to be compatible with v1.59

## v0.2.6 - September 6th, 2018

* Update replay parser to be compatible with v1.50

## v0.2.5 - May 30th, 2018

* Update replay parser to be compatible with v1.45

## v0.2.4 - April 25th, 2018

* Update replay parser to support current replays

## v0.2.3 - March 18th, 2018

* Add a `--dry-run` option that won't output JSON
* Update replay parser to support current replays

## v0.2.2 - February 14th, 2018

* Fixed several bugs surrounding parsing of the network data. More replays are now parseable

## v0.2.1 - February 1st, 2018

* If a directory argument is provided, the top level is searched for any `*.replay` files. This works around issues when the shell
  expands the glob to too many files and makes it easier to work with on Windows (which does not expand globs).

## v0.2.0 - January 31st, 2018

* Process replays in parallel using the `-m` option
* Add rudimentary network data parser. Since it's not surefire, it's not enabled by default.
* Support an older replay format

## v0.1.0 - October 26th, 2017

* Initial release
