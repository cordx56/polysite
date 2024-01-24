use polysite::*;
use std::{thread, time};

struct Wait(u64);
impl Compiler for Wait {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let sec = self.0;
        compile!({
            thread::sleep(time::Duration::from_secs(sec));
            Ok(ctx)
        })
    }
}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let builder = Builder::new(Config::default());
    builder
        .add_step([
            Rule::new("medium1")
                .set_create(["medium1"])
                .set_compiler(Wait(2).get()),
            Rule::new("long1")
                .set_create(["long1"])
                .set_compiler(Wait(3).get()),
            Rule::new("short1")
                .set_create(["short1"])
                .set_compiler(Wait(1).get()),
        ])
        .add_step([
            Rule::new("long2")
                .set_create(["long2"])
                .set_compiler(Wait(3).get()),
            Rule::new("short2")
                .set_create(["short2"])
                .set_compiler(Wait(1).get()),
            Rule::new("medium2")
                .set_create(["medium2"])
                .set_compiler(Wait(2).get()),
        ])
        .build()
        .await
        .unwrap();
}
