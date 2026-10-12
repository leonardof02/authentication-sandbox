use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Auth Server",
        description = "Experimental authentication server built with axum."
    ),
    modifiers(&BearerAuth),
    tags(
        (name = "auth", description = "Sign-up and sign-in"),
        (name = "passkeys", description = "Link passkeys and sign in with them")
    )
)]
pub struct ApiDoc;

/// Registers the `bearer_auth` scheme used by protected routes (Swagger UI's "Authorize" button).
struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
