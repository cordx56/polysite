pub mod builder;

pub use builder::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build_site() {
        let builder = Builder::new();
        let result = builder
            .add_rule(
                "compile",
                Rule::new()
                    .set_match(["src/**/*"])
                    .set_routing(routing::set_ext("txt"))
                    .set_compiler(|ctx| {
                        compiler!({
                            let hello = ctx.load("hello").await;
                            println!("hello: {:?}", hello);
                            println!(
                                "{} -> {}",
                                ctx.source().to_string_lossy(),
                                ctx.target().to_string_lossy()
                            );
                            Ok(serde_json::Value::Null)
                        })
                    }),
            )
            .await
            .add_rule(
                "hello",
                Rule::new()
                    .set_match(["src/**/*"])
                    .set_routing(routing::set_ext(".txt"))
                    .set_compiler(|ctx| {
                        compiler!({
                            let source = ctx.source();
                            Ok(Metadata::String(source.to_string_lossy().to_string()))
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
