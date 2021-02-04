use std::fmt;

use crate::{Address, TemplateAddr};

/// An in-memory representation of an app.
#[derive(PartialEq)]
pub struct App {
    /// `App`'s name
    pub name: String,

    /// `Address` of the `AppTemplate`, the App is being spawned from.
    pub template: TemplateAddr,
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self.fmt_name(&self.name);
        let template = self.fmt_template(&self.template);

        let msg = [name, template].join("\n");

        writeln!(f, "{}", msg)
    }
}

impl App {
    fn fmt_name(&self, name: &str) -> String {
        format!("Name: {}", name)
    }

    fn fmt_template(&self, addr: &TemplateAddr) -> String {
        format!("Template: {}", self.fmt_address(addr.inner()))
    }

    fn fmt_address(&self, addr: &Address) -> String {
        addr.fmt(4, 4, " ")
    }
}
