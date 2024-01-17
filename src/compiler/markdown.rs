use crate::{compiler, error::here, CompileMethod, Metadata};
use anyhow::{anyhow, Ok};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::io::Write;
use tera::{Context, Tera};

pub fn markdown_compiler(
    template_dir: impl ToString,
    template: impl ToString,
    options: Option<Options>,
) -> Box<dyn CompileMethod> {
    let template_dir = template_dir.to_string();
    let template = template.to_string();
    Box::new(move |ctx| {
        let options = options.unwrap_or(Options::all());
        let template_dir = template_dir.clone();
        let template = template.clone();
        compiler!({
            let tera = Tera::new(&template_dir)
                .map_err(|e| anyhow!("Template error on {}: {:?}", here!(), e))?;
            let body = ctx.get_source_string();
            let fm = fronma::parser::parse::<Metadata>(&body)
                .map_err(|e| anyhow!("Front matter parse error on {}: {:?}", here!(), e))?;
            let header = fm.headers;
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
            let mut metadata = ctx.metadata().clone();
            metadata
                .as_object_mut()
                .unwrap()
                .extend(header.as_object().unwrap().clone().into_iter());
            metadata
                .as_object_mut()
                .unwrap()
                .insert("body".to_string(), Metadata::String(html));
            let tera_ctx = Context::from_serialize(metadata).unwrap();
            let out = tera
                .render(template.as_ref(), &tera_ctx)
                .map_err(|e| anyhow!("Tera rendering error on {}: {:?}", here!(), e))?;
            target
                .write(out.as_bytes())
                .map_err(|e| anyhow!("Failed to write file on {}: {:?}", here!(), e))?;
            Ok(header)
        })
    })
}
