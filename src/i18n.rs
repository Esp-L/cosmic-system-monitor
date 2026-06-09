use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");
    loader
});

pub fn init() {
    let localizer = Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations));
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    if let Err(why) = localizer.select(&requested_languages) {
        eprintln!("error while loading fluent localizations: {why}");
    }
}
