pub mod builder;
pub mod error;

pub use builder::*;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use serde::Serialize;

    #[derive(Serialize, Clone, Debug)]
    struct SiteMeta {
        title: String,
    }

    #[tokio::test]
    async fn build_site() {
        let builder = Builder::new();
        let result = builder
            .add_rule(
                Rule::new("compile")
                    .set_match(["src/**/*"])
                    .set_routing(routing::set_ext("txt"))
                    .set_compiler(|ctx| {
                        compiler!({
                            let hello = ctx.wait("hello").await;
                            println!("hello: {:?}", hello);
                            println!(
                                "{} -> {}",
                                ctx.source().to_string_lossy(),
                                ctx.target().to_string_lossy()
                            );
                            Ok(Metadata::Null)
                        })
                    }),
            )
            .await
            .add_rule(
                Rule::new("hello")
                    .set_match(["src/**/*"])
                    .set_routing(routing::set_ext(".txt"))
                    .set_compiler(move |ctx| {
                        compiler!({
                            //let source = ctx.source();
                            Ok(Metadata::Null)
                        })
                    }),
            )
            .await
            .build()
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
