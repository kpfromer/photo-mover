# photo-mover

A simple script to move photos from a source directory to a destination directory based on the date the photo was taken (using EXIF metadata).

## Features

- Cross-platform (windows, macos, linux).
- Customizable folder structure.
- Handles duplicate files.
- Handles files with no EXIF metadata.

## Requirements

- ExifTool (https://exiftool.org/)

## Usage

```bash
photo-mover -s ./Images -d out/ -m move --duplicate-folder dups --no-date-folder no-date
```

## Installation

```bash
cargo install photo-mover
```