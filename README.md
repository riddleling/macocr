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

After starting the HTTP server, you can upload an image from the homepage HTML or use `curl` to send an image via the **API**

```
curl -u admin:password123 \
  -H "Accept: application/json" \
  -X POST http://localhost:80/upload \
  -F "file=@01.png"
```

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