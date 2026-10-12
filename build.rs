// `sqlx::migrate!` embeds the migrations in the binary, so rebuild when they change.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
