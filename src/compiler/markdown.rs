use crate::{compiler, error::here, CompileMethod, Metadata};
use anyhow::{anyhow, Ok};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::io::Write;

pub fn markdown_compiler(options: Option<Options>) -> Box<dyn CompileMethod> {
    Box::new(move |ctx| {
        let options = options.unwrap_or(Options::all());
        compiler!({
            let body = ctx.get_source_string();
            let fm = fronma::parser::parse::<Metadata>(&body)
                .map_err(|e| anyhow!("Front matter parse error on {}: {:?}", here!(), e))?;
            let parser = Parser::new_ext(fm.body, options);
            let mut html = String::new();
            push_html(&mut html, parser);
            let mut target = ctx.open_target().map_err(|e| {
                anyhow!(
                    "Failed to open file {} on {}: {:?}",
                    ctx.target().display(),
                    here!(),
                    e
                )
            })?;
            target
                .write(html.as_bytes())
                .map_err(|e| anyhow!("Failed to write file on {}: {:?}", here!(), e))?;
            Ok(fm.headers)
        })
    })
}
