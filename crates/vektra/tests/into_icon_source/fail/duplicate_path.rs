#[derive(vektra::IntoIconSource)]
enum AppIconName {
    #[icon(path = "icons/logo.svg", path = "icons/other.svg")]
    Logo,
}

fn main() {}
