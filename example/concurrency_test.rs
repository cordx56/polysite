use polysite::{compiler::utils::GenericCompiler, *};
use std::{thread, time};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let builder = Builder::new(Config::default());
    builder
        .add_step([
            Rule::new("medium1").set_create(["medium1"]).set_compiler(
                GenericCompiler::from(|ctx| {
                    compile!({
                        thread::sleep(time::Duration::from_secs(2));
                        Ok(ctx)
                    })
                })
                .get(),
            ),
            Rule::new("long1").set_create(["long1"]).set_compiler(
                GenericCompiler::from(|ctx| {
                    compile!({
                        thread::sleep(time::Duration::from_secs(3));
                        Ok(ctx)
                    })
                })
                .get(),
            ),
            Rule::new("short1").set_create(["short1"]).set_compiler(
                GenericCompiler::from(|ctx| {
                    compile!({
                        thread::sleep(time::Duration::from_secs(1));
                        Ok(ctx)
                    })
                })
                .get(),
            ),
        ])
        .add_step([
            Rule::new("medium2").set_create(["medium2"]).set_compiler(
                GenericCompiler::from(|ctx| {
                    compile!({
                        thread::sleep(time::Duration::from_secs(2));
                        Ok(ctx)
                    })
                })
                .get(),
            ),
            Rule::new("long2").set_create(["long2"]).set_compiler(
                GenericCompiler::from(|ctx| {
                    compile!({
                        thread::sleep(time::Duration::from_secs(3));
                        Ok(ctx)
                    })
                })
                .get(),
            ),
            Rule::new("short2").set_create(["short2"]).set_compiler(
                GenericCompiler::from(|ctx| {
                    compile!({
                        thread::sleep(time::Duration::from_secs(1));
                        Ok(ctx)
                    })
                })
                .get(),
            ),
        ])
        .build()
        .await
        .unwrap();
}
