/// This lib does not allow you to use rsx to define meta tags, instead you must use the Tag enum. The reason is because otherwise the consumer would assume that it would function just like rsx which would be incorrect b/c we do not have access to the dioxus rendering virtual DOM and render cycle. Pretending like we do would just make this lib brittle unmaintainable. Thus we have the Tag enum to explicitly define what is available and can predictibly be used.
use std::fmt;

use web_sys::{wasm_bindgen::JsValue, Document, Element, HtmlCollection, HtmlHeadElement};

#[derive(Clone, PartialEq, Copy)]
pub struct Style<'a> {
    /// id is used to determine if the non-unique style tag should be removed or updated on a re-render
    id: &'a str,
    body: &'a str,
}

#[derive(Clone, PartialEq, Copy)]
pub struct Link<'a> {
    /// id is used to determine if the non-unique link tag should be removed or updated on a re-render
    id: &'a str,
    rel: &'a str,
    href: &'a str,
}

#[derive(Clone, PartialEq)]
pub struct Script<'a> {
    /// id is used to determine if the non-unique script tag should be removed or updated on a re-render
    id: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    body: Option<&'a str>,
}

#[derive(Clone, PartialEq)]
pub enum Tag<'a> {
    /// (name, content)
    Meta(&'a str, &'a str),
    /// content
    Title(&'a str),
    Style(Style<'a>),
    Link(Link<'a>),
    Script(Script<'a>),
    /// href
    Base(&'a str),
}

impl<'a> Tag<'a> {
    const ID_BASE: &'static str = "dioxus-head-tag--";

    pub fn get_id(&self, id: &str) -> String {
        format!("{}{}", Tag::ID_BASE, id)
    }

    fn set_attrs(&self, el: &Element, attrs: &Vec<(&str, &str)>) -> Result<(), JsValue> {
        for (name, value) in attrs {
            match el.set_attribute(name, value) {
                Ok(_) => (),
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    fn create_and_mount(
        &self,
        body: Option<&str>,
        attrs: Option<&Vec<(&str, &str)>>,
        doc: &Document,
        head: &HtmlHeadElement,
    ) -> Result<(), JsValue> {
        let name = &self.to_string();
        let el = doc.create_element(name)?;
        if let Some(body) = body {
            el.set_inner_html(body);
        }
        if let Some(attrs) = attrs {
            self.set_attrs(&el, attrs)?;
        }
        head.append_child(&el)?;
        Ok(())
    }

    fn get_and_update(
        &self,
        body: Option<&str>,
        attrs: Option<&Vec<(&str, &str)>>,
        collection: HtmlCollection,
        doc: &Document,
        head: &HtmlHeadElement,
    ) -> Result<(), JsValue> {
        collection.get_with_index(0).map_or_else(
            || self.create_and_mount(body, attrs, doc, head),
            |el| {
                if let Some(body) = body {
                    el.set_inner_html(body);
                }
                Ok(())
            },
        )?;
        Ok(())
    }

    pub fn unmount(&self, doc: &Document) -> Result<(), JsValue> {
        let tag_name = &self.to_string();

        match &self {
            Tag::Title(_) | Tag::Base(_) => {
                let collection = doc.get_elements_by_tag_name(tag_name);
                let len = collection.length();

                for i in 0..len {
                    if let Some(el) = collection.get_with_index(i) {
                        doc.remove_child(&el)?;
                    }
                }

                Ok(())
            }
            Tag::Meta(name, _) => {
                let collection = doc.get_elements_by_tag_name(tag_name);
                let len = collection.length();

                for i in 0..len {
                    if let Some(el) = collection
                        .get_with_index(i)
                        .filter(|el| el.get_attribute("name").as_deref() == Some(name))
                    {
                        doc.remove_child(&el)?;
                    }
                }

                Ok(())
            }
            Tag::Style(Style { id, .. })
            | Tag::Link(Link { id, .. })
            | Tag::Script(Script { id, .. }) => {
                let id = &self.get_id(id);

                if let Some(el) = doc.get_element_by_id(id) {
                    el.remove();
                }

                Ok(())
            }
        }
    }

    pub fn mount(&self, doc: &Document, head: &HtmlHeadElement) -> Result<(), JsValue> {
        let tag_name = &self.to_string();

        match self {
            // Is defined as unique by it only allowed to be rendered in the document one time.
            // If the tag is not rendered, it will render it.
            // If the tag exists already, it will update it.
            // If any duplications of the tag exist they will be removed.
            Tag::Title(body) | Tag::Base(body) => {
                let collection = doc.get_elements_by_tag_name(tag_name);
                let len = collection.length();
                let body = Some(body);

                match len {
                    0 => self.create_and_mount(body.copied(), None, doc, head),
                    1 => self.get_and_update(body.copied(), None, collection, doc, head),
                    _ => {
                        // As a precaustion remove an additional tags that exist.
                        for i in 1..len {
                            if let Some(el) = collection.get_with_index(i) {
                                doc.remove_child(&el)?;
                            }
                        }

                        self.get_and_update(body.copied(), None, collection, doc, head)
                    }
                }
            }
            Tag::Meta(name, content) => {
                let attrs = [("name", name), ("content", content)];

                let attrs: Vec<(&str, &str)> = attrs.iter().map(|i| (i.0, *i.1)).collect();

                let collection = doc.get_elements_by_tag_name(tag_name);
                let len = collection.length();

                match len {
                    0 => self.create_and_mount(None, Some(&attrs), doc, head),
                    1 => self.get_and_update(None, Some(&attrs), collection, doc, head),
                    _ => {
                        // As a precaustion remove an additional tags that exist.
                        for i in 1..len {
                            if let Some(el) = collection.get_with_index(i) {
                                doc.remove_child(&el)?;
                            }
                        }

                        self.get_and_update(None, Some(&attrs), collection, doc, head)
                    }
                }
            }
            Tag::Style(Style { id, body }) => {
                let id = &self.get_id(id);

                if let Some(el) = doc.get_element_by_id(id) {
                    // update
                    el.set_inner_html(body);
                    Ok(())
                } else {
                    // create
                    self.create_and_mount(Some(body), None, doc, head)
                }
            }
            Tag::Link(Link { id, rel, href }) => {
                let id = &self.get_id(id);
                let attrs: Vec<(&str, &str)> = vec![("id", id), ("rel", rel), ("href", href)];

                if let Some(el) = doc.get_element_by_id(id) {
                    // update
                    self.set_attrs(&el, &attrs)
                } else {
                    // create
                    self.create_and_mount(None, Some(&attrs), doc, head)
                }
            }
            Tag::Script(Script { id, attrs, body }) => {
                let id = &self.get_id(id);

                if let Some(el) = doc.get_element_by_id(id) {
                    // update
                    if attrs.is_empty() {
                        self.set_attrs(&el, attrs)?;
                    }

                    if let Some(body) = body {
                        el.set_inner_html(body);
                    }

                    Ok(())
                } else {
                    // create
                    self.create_and_mount(body.as_ref().copied(), Some(attrs), doc, head)
                }
            }
        }
    }
}

impl<'a> fmt::Display for Tag<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Tag::Meta(_, _) => write!(f, "meta"),
            Tag::Title(_) => write!(f, "title"),
            Tag::Style(_) => write!(f, "style"),
            Tag::Script(_) => write!(f, "script"),
            Tag::Link(_) => write!(f, "link"),
            Tag::Base(_) => write!(f, "base"),
        }
    }
}
