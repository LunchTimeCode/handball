use maud::{Markup, html};

pub struct NavBarMenu {
    title: String,
    items: Vec<Markup>,
}

impl NavBarMenu {
    pub fn new(title: String, items: Vec<Markup>) -> Self {
        Self { title, items }
    }

    pub fn render(&self) -> Markup {
        html! {
           details {
               summary {
                   (self.title)
               }
               ul."bg-base-100 rounded-t-none p-2" {
                   @for item in &self.items {
                       li {
                           (item)
                       }
                   }
               }
           }
        }
    }
}

pub struct Navbar {
    title: String,
    items: Vec<Markup>,
}

impl Navbar {
    pub fn new(title: String, items: Vec<Markup>) -> Self {
        Self { title, items }
    }

    pub fn render(&self) -> Markup {
        html! {
                        div."navbar bg-base-100 shadow-sm" {
                            div."flex-1" {
                                a."btn btn-ghost text-xl" {
                                    (self.title)
                                }
                            }
                            div."flex-none" {
                                ul."menu menu-horizontal px-1" {
                                    @for item in &self.items {
                                        li {
                                            (item)
                                        }
                                }
                            }
                        }
                    }

        }
    }
}
