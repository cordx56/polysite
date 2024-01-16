use crate::RoutingMethod;

pub fn set_ext(ext: impl ToString) -> Box<dyn RoutingMethod> {
    let ext = ext.to_string();
    Box::new(move |src| {
        let mut src = src.clone();
        src.set_extension(&ext);
        src
    })
}
