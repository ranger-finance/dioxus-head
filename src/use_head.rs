use crate::{hooks::use_effect_with_cleanup, tag::Tag};

pub fn use_head(tags: Vec<Tag<'static>>) {
    use_effect_with_cleanup({
        move || {
            let window = web_sys::window()
                .expect("window was not found, Head should only be used in the browser");
            let doc = window.document().expect("document was not found");
            let head = doc.head().expect("head was not found");

            let _: Vec<Result<(), ()>> = tags
                .iter()
                .map(|tag| {
                    tag.mount(&doc, &head).map_err(|err| {
                        tracing::info!("tag mount error for tag {}: {:?}", tag.to_string(), err)
                    })
                })
                .collect();

            let tags = tags.clone();

            move || {
                let _: Vec<Result<(), ()>> = tags
                    .iter()
                    .map(|tag| {
                        tag.unmount(&doc).map_err(|err| {
                            tracing::info!(
                                "tag unmount error for tag {}: {:?}",
                                tag.to_string(),
                                err
                            )
                        })
                    })
                    .collect();
            }
        }
    });
}
