use clap::{Parser};
use infer;
use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::{fs, io::{self, Write}, path::Path};
use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_vision::{
    VNImageRequestHandler, VNRecognizeTextRequest, 
    VNRecognizeTextRequestRevision3, VNRequest, 
    VNRequestTextRecognitionLevel
};
use objc2_foundation::{
    NSArray, NSData, NSDictionary
};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Request}, 
    http::{HeaderMap, StatusCode}, 
    middleware::{self, Next}, 
    response::{Html, IntoResponse, Response}, 
    routing::{get, post}, 
    Json, 
    Router
};
use base64::{Engine as _, engine::general_purpose};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use uuid::Uuid;
use serde::Serialize;
use regex::Regex;


// app version
const VERSION: &str = env!("CARGO_PKG_VERSION");
// upload dir name
const UPLOAD_DIR_NAME: &str = "macocr_uploads";


/// OCR Tool using Vision Framework API
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input files
    #[arg(required(false))]
    files: Vec<String>,

    /// OCR and export text files
    #[arg(short('o'), long)]
    ocr: bool,

    /// Run HTTP Server
    #[arg(short('s'), long)]
    server: bool, 

    /// HTTP Basic Auth (username:password)
    #[arg(short('a'), long, default_value = "")]
    auth: String,

    /// HTTP port number
    #[arg(short('p'), long, default_value_t = 8000)]
    port: u32,
}

// Upload Json Response
#[derive(Serialize)]
struct UploadResponse {
    success: bool,
    message: String,
    ocr_result: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.ocr { 
        for file in args.files {
            if is_image(&file) {
                if let Ok(text) = get_ocr_result(&file) {
                    if let Some(stem) = Path::new(&file).file_stem().and_then(|s| s.to_str()) {
                        let text_file = format!("{}{}", stem, ".txt");
                        if let Ok(_) = export_text_file(&text, &text_file) {
                            println!("{} --> {}", file, text_file);
                        }
                    }
                }
            }
        }
    }

    if args.server {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
                }),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        let upload_dir = std::env::temp_dir().join(UPLOAD_DIR_NAME);
        std::fs::create_dir_all(&upload_dir).unwrap();

        let app = Router::new()
        .route("/", get(show_form))
        .route("/upload", post(upload_file))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            100 * 1024 * 1024, /* 100mb */
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http());

        let app = if !args.auth.is_empty() && is_valid_auth_format(&args.auth) {
            print!("      Auth: ");
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true)).unwrap();
            writeln!(&mut stdout, "{}", args.auth).unwrap();
            stdout.reset().unwrap();

            let (username, password) = args.auth.split_once(':').unwrap();
            let username = username.to_string();
            let password = password.to_string();

            app.layer(middleware::from_fn(move |headers, request, next| {
                basic_auth_middleware_with_params(headers, request, next, username.clone(), password.clone())
            }))
        } else {
            app
        };

        let addr = format!("0.0.0.0:{}", args.port.to_string());

        print!("   Address: ");
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true)).unwrap();
        writeln!(&mut stdout, "http://{}", addr).unwrap();
        stdout.reset().unwrap();

        print!("Upload dir: ");
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true)).unwrap();
        writeln!(&mut stdout, "{}", upload_dir.to_str().unwrap()).unwrap();
        stdout.reset().unwrap();
        println!("");
                
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

fn is_image(path: &str) -> bool {
    let data = fs::read(path);
    match data {
        Ok(data) => infer::is_image(&data),
        Err(_) => false
    }
}

fn get_ocr_result(path: &str) -> io::Result<String> {
    let bytes = fs::read(path)?;
    let data = NSData::with_bytes(&bytes);

    let request = VNRecognizeTextRequest::new();
    unsafe { request.setRevision(VNRecognizeTextRequestRevision3) };
    request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
    request.setUsesLanguageCorrection(true);
    request.setAutomaticallyDetectsLanguage(true);

    let request_super: Retained<VNRequest> = unsafe { Retained::cast_unchecked(request.clone()) };
    let requests = NSArray::from_retained_slice(&[
        request_super
    ]);

    let options = NSDictionary::new();
    let handler = VNImageRequestHandler::initWithData_options(
        VNImageRequestHandler::alloc(), &data, &options
    );
        
    let _ = handler.performRequests_error(&requests);
    
    let mut result = String::new();
    if let Some(observations) = request.results() {
        for observation in observations {
            if let Some(candidate) = observation.topCandidates(1).firstObject() {
                result.push_str(&format!("{}\n", candidate.string()));
            }
        }
    }

    Ok(result)
}

fn export_text_file(contents: &str, path: &str) -> io::Result<()> {
    fs::write(path, contents)?;
    Ok(())
}

// 顯示單檔案上傳表單
async fn show_form() -> Html<String> {
    let html = format!(
        r#"
        <!doctype html>
        <html>
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>macocr</title>
        </head>
        <body>
            <h1>macocr v{}</h1>
            <form action="/upload" method="post" enctype="multipart/form-data">
                <label>
                    Choose file: 
                    <input type="file" name="file" required>
                </label>
                <br><br>
                <input type="submit" value="Upload file">
            </form>
        </body>
        </html>
        "#, 
        VERSION
    );
    Html(html)
}

// Handle single file upload – supports HTML and JSON responses
async fn upload_file(headers: HeaderMap, mut multipart: Multipart) -> impl IntoResponse {
    // Determine if the request is an API request (based on the Accept header)
    let is_api_request = headers.get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.contains("application/json"))
        .unwrap_or(false);
    
    // Get the first field
    if let Some(field) = multipart.next_field().await.unwrap() {
        let original_name = field.file_name().unwrap_or("unnamed").to_string();
        let data = field.bytes().await.unwrap();
        
        // Generate a random filename while preserving the original file extension
        let file_extension = std::path::Path::new(&original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let random_name = if file_extension.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            format!("{}.{}", Uuid::new_v4(), file_extension)
        };
        
        // Generate a storage path under the system temporary directory
        let upload_dir = std::env::temp_dir().join(UPLOAD_DIR_NAME);
        let save_path = upload_dir.join(&random_name);
        
        // Write to file
        match std::fs::File::create(&save_path) {
            Ok(mut file) => {
                match file.write_all(&data) {
                    Ok(_) => {
                        let mut success = false;
                        let mut title = "❌ The file type is not an image".to_string();
                        let mut message = "The file type is not an image".to_string();
                        let mut ocr_result= "".to_string();

                        if let Some(path_str) = save_path.to_str() {
                            if is_image(&path_str) {
                                if let Ok(text) = get_ocr_result(&path_str) {
                                    ocr_result = text;
                                    message = "File uploaded successfully".to_string();
                                    title = "OCR Result:".to_string();
                                    success = true;
                                }
                            } 
                        } 
                    
                        if is_api_request {
                            Json(UploadResponse {
                                success: success,
                                message: message.to_string(),
                                ocr_result: ocr_result,
                            }).into_response()
                        } else {
                            Html(format!(
                                r#"
                                <!doctype html>
                                <html>
                                <head>
                                    <meta charset="utf-8">
                                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                                    <title>OCR Result</title>
                                </head>
                                <body>
                                    <h1>{}</h1>
                                    <p>{}</p>
                                </body>
                                </html>
                                "#,
                                title, ocr_result.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\n", "<br>")
                            )).into_response()
                        }
                    }
                    Err(_) => {
                        if is_api_request {
                            Json(UploadResponse {
                                success: false,
                                message: "Failed to write file".to_string(),
                                ocr_result: "".to_string()
                            }).into_response()
                        } else {
                            Html(r#"
                                <!doctype html>
                                <head>
                                    <meta charset="utf-8">
                                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                                    <title>Error</title>
                                </head>
                                <html><body>
                                    <h1>❌ Failed to write file.</h1>
                                </body></html>
                            "#.to_string()).into_response()
                        }
                    }
                }
            }
            Err(_) => {
                if is_api_request {
                    Json(UploadResponse {
                        success: false,
                        message: "Unable to create file".to_string(),
                        ocr_result: "".to_string()
                    }).into_response()
                } else {
                    Html(r#"
                        <!doctype html>
                        <head>
                            <meta charset="utf-8">
                            <meta name="viewport" content="width=device-width, initial-scale=1.0">
                            <title>Error</title>
                        </head>
                        <html><body>
                            <h1>❌ Unable to create file.</h1>
                        </body></html>
                    "#.to_string()).into_response()
                }
            }
        }
    } else {
        if is_api_request {
            Json(UploadResponse {
                success: false,
                message: "No file received".to_string(),
                ocr_result: "".to_string()
            }).into_response()
        } else {
            Html(r#"
                <!doctype html>
                <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Error</title>
                </head>
                <html><body>
                    <h1>❌ No file received</h1>
                </body></html>
            "#.to_string()).into_response()
        }
    }
}

fn is_valid_auth_format(input: &str) -> bool {
    let re = Regex::new(r"^[^:]+:[^:]+$").unwrap();
    re.is_match(input)
}

// Basic Auth middleware
async fn basic_auth_middleware_with_params(
    headers: HeaderMap,
    request: Request,
    next: Next,
    username: String,
    password: String,
) -> Result<Response, StatusCode> {
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                let encoded = &auth_str[6..]; // Remove the 'Basic ' prefix
                if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(encoded) {
                    if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                        // Split the username and password
                        if let Some((user, pass)) = decoded_str.split_once(':') {
                            if user == username && pass == password {
                                // Authentication successful, proceeding with the request
                                return Ok(next.run(request).await);
                            }
                        }
                    }
                }
            }
        }
    }

    // Authentication failed, return 401 and request authentication
    let mut response = Response::new("Authentication failed: A valid username and password are required.".into());
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    response.headers_mut().insert(
        "WWW-Authenticate",
        "Basic realm=\"File Upload Server\"".parse().unwrap(),
    );
    
    Ok(response)
}
