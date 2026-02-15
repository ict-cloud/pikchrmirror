use crate::app::message::Message;
use crate::app::update;
use crate::app::MirrorApp;
use iced::highlighter;
use iced::widget::text_editor;

#[test]
fn test_message_new_file() {
    let message = Message::NewFile;
    assert_eq!(format!("{:?}", message), "NewFile");
}

#[test]
fn test_message_open_file() {
    let message = Message::OpenFile;
    assert_eq!(format!("{:?}", message), "OpenFile");
}

#[test]
fn test_message_save_file() {
    let message = Message::SaveFile;
    assert_eq!(format!("{:?}", message), "SaveFile");
}

#[test]
fn test_message_theme_selected() {
    let message = Message::ThemeSelected(highlighter::Theme::SolarizedDark);
    assert_eq!(format!("{:?}", message), "ThemeSelected(SolarizedDark)");
}

#[test]
fn test_message_file_saved() {
    let message = Message::FileSaved;
    assert_eq!(format!("{:?}", message), "FileSaved");
}

#[test]
fn test_mirror_app_default_initialization() {
    let app = MirrorApp::default();

    let initial_text = r##"arrow right 200% "Markdown" "HTML"
box rad 10px "Markdown" "(markdown.c)" fit
arrow right 200% "HTML" "pikchr"
box rad 10px "HTML" "(pikchr.c)" fit
arrow right 200% "pikchr" "SVG"
box rad 10px "pikchr" "(cgi/pikchr.c)" fit
"##;

    assert_eq!(
        app.text, initial_text,
        "Initial text should match expected value."
    );
    assert_eq!(
        app.content.text(),
        initial_text,
        "Content should reflect initial text."
    );

    // Assert that SVG is generated and not empty.
    // A more robust test would parse the SVG to ensure correctness,
    // but for a default initialization, checking for non-empty output is a good start.
    assert!(!app.svg.is_empty(), "SVG output should not be empty.");
    assert!(
        app.svg.contains("<svg"),
        "SVG output should contain an <svg> tag."
    );

    assert!(
        app.error.is_empty(),
        "Error should be empty for valid initial text."
    );
    assert_eq!(
        app.theme,
        highlighter::Theme::InspiredGitHub,
        "Default theme should be InspiredGitHub."
    );
}

// Helper function to create a default app for each test
fn setup_app() -> MirrorApp {
    MirrorApp::default()
}

#[test]
fn test_update_new_file() {
    let mut app = setup_app();
    // Simulate having some content before creating a new file
    app.text = String::from("existing content");
    app.content = text_editor::Content::with_text("existing content");
    app.svg = String::from("<svg>...</svg>");
    app.error = String::from("some error");

    let _task = update::update(&mut app, Message::NewFile);

    assert!(app.text.is_empty());
    assert!(app.content.text().is_empty());
    assert!(
        !app.svg.is_empty(),
        "SVG should be generated for empty input (empty SVG)"
    );
    assert!(
        app.error.is_empty(),
        "Error should be empty for empty input"
    );
    //assert_eq!(task, Task::none());
}

#[test]
fn test_update_open_file() {
    let mut app = setup_app(); // State should not change for OpenFile
    let _task = update::update(&mut app, Message::OpenFile);
    //assert_eq!(task, Task::none());
}

#[test]
fn test_update_save_file() {
    let mut app = setup_app();
    // Ensure app.svg has some content to be saved
    app.svg = String::from("<svg>test</svg>");

    let _task = update::update(&mut app, Message::SaveFile);

    // We can't easily inspect the `actions::save_file` call without mocking,
    // but we can assert that a Task other than `Task::none()` is returned.
    //assert_ne!(task, Task::none());
}

#[test]
fn test_update_theme_selected() {
    let mut app = setup_app();
    let initial_theme = app.theme;
    let new_theme = highlighter::Theme::SolarizedDark;

    let _task = update::update(&mut app, Message::ThemeSelected(new_theme));

    assert_eq!(app.theme, new_theme);
    assert_ne!(app.theme, initial_theme);
    //assert_eq!(task, Task::none());
}

#[test]
fn test_update_file_saved() {
    let mut app = setup_app(); // State should not change for FileSaved
    let _task = update::update(&mut app, Message::FileSaved);
    //assert_eq!(task, Task::none());
}

#[test]
fn test_theme_switching_single() {
    let mut app = setup_app();
    let initial_theme = app.theme;

    // Switch to SolarizedDark
    let _task = update::update(
        &mut app,
        Message::ThemeSelected(highlighter::Theme::SolarizedDark),
    );

    assert_eq!(app.theme, highlighter::Theme::SolarizedDark);
    assert_ne!(app.theme, initial_theme);
}

#[test]
fn test_theme_switching_multiple() {
    let mut app = setup_app();
    let themes = vec![
        highlighter::Theme::SolarizedDark,
        highlighter::Theme::InspiredGitHub,
    ];

    for theme in themes {
        let _task = update::update(&mut app, Message::ThemeSelected(theme));
        assert_eq!(app.theme, theme, "Theme should be updated to {:?}", theme);
    }
}

#[test]
fn test_theme_switching_preserves_content() {
    let mut app = setup_app();
    let original_text = app.text.clone();
    let original_svg = app.svg.clone();
    let original_error = app.error.clone();

    // Switch theme
    let _task = update::update(
        &mut app,
        Message::ThemeSelected(highlighter::Theme::SolarizedDark),
    );

    // Content should remain unchanged
    assert_eq!(
        app.text, original_text,
        "Text should be preserved after theme switch"
    );
    assert_eq!(
        app.svg, original_svg,
        "SVG should be preserved after theme switch"
    );
    assert_eq!(
        app.error, original_error,
        "Error should be preserved after theme switch"
    );
}

#[test]
fn test_theme_switching_all_available_themes() {
    let mut app = setup_app();

    // Test all available themes
    for &theme in highlighter::Theme::ALL {
        let _task = update::update(&mut app, Message::ThemeSelected(theme));
        assert_eq!(
            app.theme, theme,
            "Theme should be correctly updated to {:?}",
            theme
        );
    }
}

#[test]
fn test_theme_message_creation() {
    // Test that ThemeSelected message is created correctly
    let theme = highlighter::Theme::SolarizedDark;
    let message = Message::ThemeSelected(theme);

    match message {
        Message::ThemeSelected(t) => {
            assert_eq!(t, theme, "Theme in message should match the input theme");
        }
        _ => panic!("Message should be ThemeSelected variant"),
    }
}

#[test]
fn test_theme_affects_app_appearance() {
    let mut app = setup_app();
    let dark_theme = highlighter::Theme::SolarizedDark;
    let light_theme = highlighter::Theme::InspiredGitHub;

    // Set dark theme
    let _ = update::update(&mut app, Message::ThemeSelected(dark_theme));
    assert!(app.theme.is_dark(), "SolarizedDark should be a dark theme");

    // Set light theme
    let _ = update::update(&mut app, Message::ThemeSelected(light_theme));
    assert!(
        !app.theme.is_dark(),
        "InspiredGitHub should not be a dark theme"
    );
}
