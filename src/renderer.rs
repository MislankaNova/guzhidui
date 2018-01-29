use iron::error::IronError;
use iron::status;
use tera::{Tera, Context};

use std::error;

use iron_archivist::{Entry, Renderer, RenderResult};

// A wrapper around the Tera template engine
pub struct TeraRenderer(pub Tera);

impl Renderer for TeraRenderer {
    fn render_dir(&self, path_str: &str, entries: &[Entry]) -> RenderResult {
        let mut context = Context::new();
        context.add(
            "path",
            &path_str
        );
        context.add(
            "entries",
            &entries.iter().map(entry_to_context).collect::<Vec<_>>()
        );
        self.0.render("dir.html", &context)
            .map_err(raise_internal_server_error)
    }

    fn render_verbatim(&self, path_str: &str, content: &str) -> RenderResult {
        let mut context = Context::new();
        context.add(
            "path",
            &path_str
        );
        context.add(
            "content",
            &content
        );
        self.0.render("verbatim.html", &context)
            .map_err(raise_internal_server_error)
    }

    fn render_markdown(&self, path_str: &str, content: &str) -> RenderResult {
        let mut context = Context::new();
        context.add(
            "path",
            &path_str
        );
        context.add(
            "content",
            &content
        );
        self.0.render("markdown.html", &context)
            .map_err(raise_internal_server_error)
    }

    fn render_error(
            &self,
            path_str: &str,
            code: usize,
            message: &str) -> RenderResult {
        let mut context = Context::new();
        context.add(
            "path",
            &path_str
        );
        context.add(
            "code",
            &code
        );
        context.add(
            "message",
            &message
        );
        self.0.render("error.html", &context)
            .map_err(raise_internal_server_error)
    }
}

fn raise_internal_server_error<E: 'static + error::Error + Send>(e: E)
        -> IronError {
    IronError::new(e, status::InternalServerError)
}

fn entry_to_context(entry: &Entry) -> Context {
    let mut context = Context::new();
    context.add("is_dir", &entry.is_dir);
    context.add("file_name", &entry.file_name);
    context.add("modified", &entry.modified);
    context
}
