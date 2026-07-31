use vektra::{IconSource, IntoIconSource};

#[derive(Debug, Clone, Copy, vektra::IntoIconSource)]
enum AppIconName {
    Logo,
    FavoriteFilled,
    #[icon(path = "icons/heart.svg")]
    Favorite,
}

#[test]
fn derive_generates_snake_case_paths() {
    assert_eq!(
        AppIconName::Logo.into_icon_source().path(),
        "icons/logo.svg"
    );
    assert_eq!(
        AppIconName::Favorite.into_icon_source().path(),
        "icons/heart.svg"
    );
    assert_eq!(
        AppIconName::FavoriteFilled.into_icon_source().path(),
        "icons/favorite_filled.svg"
    );
}

#[test]
fn derive_uses_explicit_path_override() {
    assert_eq!(
        AppIconName::Favorite.into_icon_source().path(),
        "icons/heart.svg"
    );
}

#[test]
fn manual_into_icon_source_impl_still_works() {
    #[derive(Debug, Clone, Copy)]
    enum ManualIcon {
        Logo,
    }

    impl IntoIconSource for ManualIcon {
        fn into_icon_source(self) -> IconSource {
            match self {
                Self::Logo => IconSource::asset("icons/manual_logo.svg"),
            }
        }
    }

    assert_eq!(
        ManualIcon::Logo.into_icon_source().path(),
        "icons/manual_logo.svg"
    );
}

#[test]
fn derive_compile_failures_are_reported() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/into_icon_source/fail/*.rs");
}
