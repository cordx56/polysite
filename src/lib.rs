mod rule;
mod site;

pub use rule::Rule;
pub use site::builder::SiteBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build_site() {
        let builder = SiteBuilder::new();
        let result = builder
            .add_rule(
                "hello",
                Rule::new().set_compiler(|ctx, rule| {
                    println!("Hello, Compiler!");
                    Ok(())
                }),
            )
            .await
            .add_rule(
                "world",
                Rule::new().set_compiler(|ctx, rule| {
                    //ctx.lock().await.load("hello").await;
                    println!("world!");
                    Ok(())
                }),
            )
            .await
            .build()
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
