# macocr

An OCR Tool using Apple's Vision Framework API.

## Command Line Arguments

```
OCR Tool using Vision Framework API

Usage: macocr [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Input files

Options:
  -o, --ocr          OCR and export text files
  -s, --server       Run HTTP Server
  -a, --auth <AUTH>  HTTP Basic Auth (username:password) [default: ]
  -p, --port <PORT>  HTTP port number [default: 8000]
  -h, --help         Print help
  -V, --version      Print version
```

## How to use

### Read images and perform OCR, then output the result to stdout

```
macocr *.png
```

### Read images and perform OCR, then output the result to text files

```
macocr -o *.png
```

### Start the OCR HTTP server and specify the HTTP port

```
macocr -s -p 80
```

### Start the OCR HTTP server and configure HTTP Basic Auth

```
macocr -s -a admin:password123 -p 80
```

After starting the HTTP server, you can upload an image from the homepage HTML or use `curl` to send an image via the `upload` API:

```
curl -u admin:password123 \
  -H "Accept: application/json" \
  -X POST http://localhost:80/upload \
  -F "file=@01.png"
```

The JSON response looks like this:

```json
{
    "success": true,
    "message": "File uploaded successfully",
    "ocr_result": "Hello\nWorld\n",
    "image_width": 1247,
    "image_height": 648,
    "ocr_boxes": [
        {
            "text": "Hello",
            "x": 429.5830268255751,
            "y": 267.7961617530676,
            "w": 201.98298336909374,
            "h": 72.4076766967774,
            "rect": {
                "top_left_x": 429.5830268255751,
                "top_left_y": 268.2039872230384,
                "top_right_x": 631.4207561940295,
                "top_right_y": 267.7961617530676,
                "bottom_right_x": 631.5660101946688,
                "bottom_right_y": 339.7960129798743,
                "bottom_left_x": 429.7282808262144,
                "bottom_left_y": 340.203838449845
            }
        },
        {
            "text": "World",
            "x": 421.6618595339102,
            "y": 417.99999973333337,
            "w": 251.79807692307696,
            "h": 80.0,
            "rect": {
                "top_left_x": 421.6618595339102,
                "top_left_y": 417.99999973333337,
                "top_right_x": 673.4599364569872,
                "top_right_y": 417.99999973333337,
                "bottom_right_x": 673.4599364569872,
                "bottom_right_y": 497.99999973333337,
                "bottom_left_x": 421.6618595339102,
                "bottom_left_y": 497.99999973333337
            }
        }
    ]
}
```

`image_width` and `image_height` represent the width and height of the image (in px),
`x` and `y` represent the top-left origin of the text bounding box (in px),
`w` and `h` represent the width and height of the text bounding box (in px).


## Installation

### Install by cargo

```
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install macocr
cargo install macocr
macocr -h
```

## Features

- Directly invoke Apple's Vision Framework API for OCR
- Command-line mode: allows batch processing of image files and exports OCR results as TXT files
- HTTP server mode: provides a web interface to upload images and return OCR results
- Supports both HTML form upload and API interfaces
- Configurable HTTP Basic Auth authentication
- The maximum upload image size is 100 MB

## Use cases

- macOS users need to perform batch OCR processing
- Applications that need to integrate OCR functionality via API


## License

MIT License