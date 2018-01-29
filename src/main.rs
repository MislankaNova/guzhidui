#[macro_use]
extern crate clap;
extern crate iron;
extern crate iron_archivist;
extern crate mount;
extern crate router;
extern crate tera;

use clap::{App, SubCommand, Arg};
use iron::prelude::*;
use iron::status;
use iron::headers::ContentType;
use iron::middleware::AfterMiddleware;
use iron::modifiers::Header;
use iron_archivist::*;
use mount::Mount;
use router::Router;
use tera::Tera;

use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;

mod renderer;
use renderer::TeraRenderer;

static FOOTER_HTML:   &'static str = include_str!("../template/footer.html");
static HEADER_HTML:   &'static str = include_str!("../template/header.html");
static DIR_HTML:      &'static str = include_str!("../template/dir.html");
static MARKDOWN_HTML: &'static str = include_str!("../template/markdown.html");
static VERBATIM_HTML: &'static str = include_str!("../template/verbatim.html");
static ERROR_HTML:    &'static str = include_str!("../template/error.html");
static STYLE_CSS:     &'static str = include_str!("../style.css");

// If Iron runs into an error for whatever reason
// Handle it here
struct UnknownHandler;

impl AfterMiddleware for UnknownHandler {
    fn catch(&self, _: &mut Request, _: IronError)
            -> IronResult<Response> {
        // Hard code the error page here
        Ok(Response::with((
            include_str!("../template/unknown-error.html"),
            status::InternalServerError,
            Header(ContentType::html())
        )))
    }
}

// Serve our stylesheet here
struct StylesheetHandler(String);

impl iron::Handler for StylesheetHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((
            self.0.as_str(),
            status::Ok,
            Header(ContentType::plaintext())
        )))
    }
}

impl Default for StylesheetHandler {
    fn default() -> Self {
        StylesheetHandler(String::from(STYLE_CSS))
    }
}

fn write_to_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
    println!("Extracting {}.", path.as_ref().display());
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn extract_default<P: AsRef<Path>>(dest: P) -> io::Result<()> {
    fs::create_dir(&dest)?;
    fs::create_dir(dest.as_ref().join("template"))?;
    fs::create_dir(dest.as_ref().join("style"))?;
    write_to_file(dest.as_ref().join("template/footer.html"), FOOTER_HTML)?;
    write_to_file(dest.as_ref().join("template/header.html"), HEADER_HTML)?;
    write_to_file(dest.as_ref().join("template/dir.html"), DIR_HTML)?;
    write_to_file(dest.as_ref().join("template/markdown.html"), MARKDOWN_HTML)?;
    write_to_file(dest.as_ref().join("template/verbatim.html"), VERBATIM_HTML)?;
    write_to_file(dest.as_ref().join("template/error.html"), ERROR_HTML)?;
    write_to_file(dest.as_ref().join("style/style.css"), STYLE_CSS)?;
    println!("Done.");
    Ok(())
}

fn load_templates_from_dir<P: AsRef<Path>>(dir: P) -> tera::Result<Tera> {
    let mut tera = Tera::default();
    tera.add_template_files(vec![
        (dir.as_ref().join("footer.html"), Some("footer.html")),
        (dir.as_ref().join("header.html"), Some("header.html")),
        (dir.as_ref().join("dir.html"), Some("dir.html")),
        (dir.as_ref().join("markdown.html"), Some("markdown.html")),
        (dir.as_ref().join("verbatim.html"), Some("verbatim.html")),
        (dir.as_ref().join("error.html"), Some("error.html"))
    ])?;
    Ok(tera)
}

fn load_default_templates() -> Tera {
    let mut tera = Tera::default();
    // No need to handle error since we are using the default templates
    tera.add_raw_template(
        "footer.html",
        FOOTER_HTML
    ).unwrap();
    tera.add_raw_template(
        "header.html",
        HEADER_HTML
    ).unwrap();
    tera.add_raw_template(
        "dir.html",
        DIR_HTML
    ).unwrap();
    tera.add_raw_template(
        "markdown.html",
        MARKDOWN_HTML
    ).unwrap();
    tera.add_raw_template(
        "verbatim.html",
        VERBATIM_HTML
    ).unwrap();
    tera.add_raw_template(
        "error.html",
        ERROR_HTML
    ).unwrap();
    tera
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("silent")
             .long("silent")
             .help("Suppress command line output."))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .takes_value(true)
             .help("Load configuration from a file"))
        .arg(Arg::with_name("root")
             .short("r")
             .long("root")
             .value_name("DIR")
             .takes_value(true)
             .required_unless("config")
             .help("The directory to serve"))
        .arg(Arg::with_name("template")
             .short("t")
             .long("template")
             .value_name("DIR")
             .takes_value(true)
             .help("The directory where the template files are located"))
        .arg(Arg::with_name("style")
             .short("s")
             .long("css")
             .value_name("FILE")
             .takes_value(true)
             .help("The path to the stylesheet to be used."))
        .arg(Arg::with_name("listen")
             .short("l")
             .long("listen")
             .value_name("ADDR")
             .takes_value(true)
             .required_unless("config")
             .help("The address to listen"))
        .arg(Arg::with_name("allow-all")
             .long("allow-all")
             .help("Allow all files"))
        .arg(Arg::with_name("allowed-file-names")
             .short("a")
             .long("allow")
             .value_name("FILE")
             .multiple(true)
             .takes_value(true)
             .help("File names allowed"))
        .arg(Arg::with_name("allowed-extensions")
             .short("ext")
             .long("allow-extensions")
             .value_name("EXT")
             .multiple(true)
             .takes_value(true)
             .help("File extensions allowed"))
        .arg(Arg::with_name("blocked-file-names")
             .short("b")
             .long("block")
             .value_name("FILE")
             .multiple(true)
             .takes_value(true)
             .help("File names blocked from access"))
        .arg(Arg::with_name("markdown")
             .short("md")
             .long("markdown")
             .value_name("EXT")
             .multiple(true)
             .takes_value(true)
             .help("File extensions treated as Markdown script"))
        .subcommand(SubCommand::with_name("extract")
                    .about("Extract the default HTML templates and stylesheet.")
                    .arg(Arg::with_name("dest")
                         .index(1)
                         .value_name("DIR")
                         .takes_value(true)
                         .required(true)
                         .help("The directory where the extracted files will be placed.")))
        .get_matches();

    // If extracting then just extract and return
    if let Some(sc) = matches.subcommand_matches("extract") {
        let dest = &sc.value_of("dest").unwrap();
        println!(
            "Extracting templates and stylesheet to {}.",
            dest
        );
        if let Err(e) = extract_default(&dest) {
            eprintln!("[ERROR] Error when extracting files.");
            eprintln!("[ERROR] {}", e);
        }
        return;
    }

    // If the config argument is set then try to load the config file
    // Otherwise start with a blank config
    let mut config = if let Some(s) = matches.value_of("config") {
        match Config::load(&s) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[ERROR] Error when loading configuration.");
                eprintln!("[ERROR] {}", e);
                return;
            },
        }
    } else {
        Config::default()
    };

    // Override the value in the config with command line arguments

    // Required if no config file loaded
    if let Some(s) = matches.value_of("root") {
        config.root_dir = String::from(s);
    }
    // Required if no config file loaded
    if let Some(s) = matches.value_of("listen") {
        config.listen = String::from(s);
    }
    if matches.is_present("allow-all") {
        config.allow_all = true;
    }
    if let Some(vs) = matches.values_of("allowed-file-names") {
        config.allowed_file_names = vs.map(OsString::from).collect();
    }
    if let Some(vs) = matches.values_of("allowed-extensions") {
        config.allowed_extensions = vs.map(OsString::from).collect();
    }
    if let Some(vs) = matches.values_of("blocked-file-names") {
        config.blocked_file_names = vs.map(OsString::from).collect();
    }
    if let Some(vs) = matches.values_of("markdown") {
        config.markdown = vs.map(OsString::from).collect();
    }

    // Load the templates
    // Inline the template content here so there is no need to load during runtime
    let tera = if let Some(s) = matches.value_of("template") {
    // Load from a directory if it is specified
        match load_templates_from_dir(&s) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[ERROR] Error when loading templates.");
                eprintln!("[ERROR] {}", e);
                return;
            },
        }
    } else {
        load_default_templates()
    };

    // Prapare the stylesheet handler
    let style_handler = if let Some(s) = matches.value_of("style") {
        let mut file = match fs::File::open(&s) {
            Ok(f) => f,
            Err(e) =>  {
                eprintln!("[ERROR] Error when loading stylesheet.");
                eprintln!("[ERROR] {}", e);
                return;
            },
        };
        let mut content = String::new();
        match file.read_to_string(&mut content) {
            Ok(_) => StylesheetHandler(content),
            Err(e) =>  {
                eprintln!("[ERROR] Error when reading stylesheet.");
                eprintln!("[ERROR] {}", e);
                return;
            },
        }
    } else {
        StylesheetHandler::default()
    };

    let mut style_router = Router::new();
    style_router.get("/", style_handler, "stylesheet");

    let renderer_arc = Arc::new(TeraRenderer(tera));

    // /style/ is just the stylesheet
    // /r/ is the raw files
    // / is the files and directories
    let mut mount = Mount::new();
    mount.mount("/.style/", style_router);
    mount.mount(
        "/.r/",
        Archivist::summon_raw(&config, renderer_arc.clone())
    );
    mount.mount(
        "/",
        Archivist::summon(&config, renderer_arc.clone())
    );

    // Just throw all the errors into our handler
    let mut chain = Chain::new(mount);
    chain.link_after(UnknownHandler);

    if !matches.is_present("silent") {
        println!("Serving {}", &config.root_dir);
        println!("Listening on {}", &config.listen);
    }

    Iron::new(chain)
        .http(&config.listen)
        .unwrap();
}
