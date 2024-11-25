use polysite::*;
use std::{thread, time};

#[derive(Clone)]
struct Wait(u64);
impl Compiler for Wait {
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        let sec = self.0;
        compile!({
            thread::sleep(time::Duration::from_secs(sec));
            Ok(CompileStep::Completed(ctx))
        })
    }
}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let builder = Builder::new(Config::default());
    builder
        .add_step([
            Rule::new("medium1", Wait(2)).set_create(["medium1"]),
            Rule::new("long1", Wait(3)).set_create(["long1"]),
            Rule::new("short1", Wait(1)).set_create(["short1"]),
        ])
        .add_step([
            Rule::new("long2", Wait(3)).set_create(["long2"]),
            Rule::new("short2", Wait(1)).set_create(["short2"]),
            Rule::new("medium2", Wait(2)).set_create(["medium2"]),
        ])
        .build()
        .await
        .unwrap();
}
