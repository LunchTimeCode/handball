use maud::{Markup, html};

use super::components::navbar::{NavBarMenu, Navbar};

pub fn header() -> Markup {
    let menu = NavBarMenu::new(
        "Menu".to_string(),
        vec![
            html! {
                a href="/home" { "Home" }
            },
            html! {
                a href="/about" { "About" }
            },
            html! {
                a href="/contact" { "Contact" }
            },
        ],
    );

    let navbar = Navbar::new(
        "Handball".to_string(),
        vec![
            html! {
                a href="/dashboard" { "Dashboard" }
            },
            menu.render(),
        ],
    );

    navbar.render()
}
