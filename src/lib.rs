mod rule;
mod site;

use std::collections::HashMap;

pub use rule::Rule;
pub use site::builder::SiteBuilder;

pub type CompileResult = Result<(), ()>;
pub type Metadata = HashMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build_site() {
        let builder = SiteBuilder::new();
        let result = builder
            .add_rule(
                "hello",
                Rule::new().set_compiler(|ctx, _rule| {
                    Box::new(Box::pin(async move {
                        ctx.load("world").await;
                        println!("Hello, Compiler!");
                        Ok(())
                    }))
                }),
            )
            .await
            .add_rule(
                "world",
                Rule::new().set_compiler(|_ctx, _rule| {
                    Box::new(Box::pin(async move {
                        println!("world!");
                        Ok(())
                    }))
                }),
            )
            .await
            .build()
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
