mod auth;

#[macro_use]
extern crate rocket;

#[cfg(test)]
#[macro_use]
extern crate rstest;

use crate::auth::{OpenIDClient, User};
use anyhow::anyhow;
use openid::Options;
use rocket::{
    fs::FileServer, fs::NamedFile, http::Status, response::Redirect, serde::Serialize, Either,
    State,
};
use serde::Deserialize;

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Environment {
    #[default]
    Prod,
    Test,
}

impl Environment {
    fn from_env() -> anyhow::Result<Self> {
        match std::env::var("STAGE")?.to_lowercase().as_ref() {
            "prod" => Ok(Self::Prod),
            "test" => Ok(Self::Test),
            name => Err(anyhow!("unknown environment: {name}")),
        }
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    rocket::build()
        .manage(Environment::from_env().unwrap_or_default())
        .attach(auth::fairing())
        .mount(
            "/",
            routes![index, auth::finalize, auth::accept_invite, error_page],
        )
        .register("/", catchers![not_found])
        .launch()
        .await
        .map(|_| ())
}

#[get("/")]
async fn index(
    user: Option<User>,
    oidc_client: &State<OpenIDClient>,
    oidc_options: &State<Options>,
) -> Result<Either<Redirect, NamedFile>, Status> {
    if user.is_none() {
        let url = oidc_client.auth_url(oidc_options).to_string();
        return Ok(Either::Left(Redirect::to(url)));
    };
    NamedFile::open("dist/index.html")
        .await
        .map(Either::Right)
        .map_err(|err| {
            error!("Cannot open index file: {err}");
            Status::InternalServerError
        })
}

#[get("/error/<_page>")]
async fn error_page(_page: &str) -> Result<NamedFile, (Status, &'static str)> {
    frontend().await
}

#[catch(404)]
async fn not_found() -> Result<NamedFile, (Status, &'static str)> {
    frontend().await
}

async fn frontend() -> Result<NamedFile, (Status, &'static str)> {
    NamedFile::open("dist/index.html").await.map_err(|err| {
        error!("Cannot open index file: {err}");
        (Status::NotFound, "Not found")
    })
}
