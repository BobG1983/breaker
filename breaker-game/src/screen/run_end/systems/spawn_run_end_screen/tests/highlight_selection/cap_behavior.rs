use super::{super::helpers::*, helpers::*};
use crate::run::{
    definition::HighlightConfig,
    resources::{RunOutcome, RunStats},
};

#[test]
fn highlight_cap_reads_from_config() {
    let stats = RunStats {
        highlights: make_highlights(6),
        ..Default::default()
    };
    let mut app = test_app_with_stats(RunOutcome::Won, stats);
    app.insert_resource(HighlightConfig {
        highlight_cap: 4,
        ..Default::default()
    });
    app.update();

    let texts = collect_texts(&mut app);
    let highlight_count = texts.iter().filter(|t| is_highlight_text(t)).count();
    assert_eq!(
        highlight_count, 4,
        "expected 4 highlights when HighlightConfig.highlight_cap = 4, got {highlight_count} in texts: {texts:?}"
    );
}

#[test]
fn highlight_cap_falls_back_to_three_without_config() {
    let stats = RunStats {
        highlights: make_highlights(6),
        ..Default::default()
    };
    let mut app = test_app_with_stats(RunOutcome::Won, stats);
    // Deliberately do NOT insert HighlightConfig.
    app.update();

    let texts = collect_texts(&mut app);
    let highlight_count = texts.iter().filter(|t| is_highlight_text(t)).count();
    assert_eq!(
        highlight_count, 3,
        "expected 3 highlights as fallback without HighlightConfig, got {highlight_count} in texts: {texts:?}"
    );
}

#[test]
fn highlight_cap_shows_fewer_when_fewer_exist() {
    let stats = RunStats {
        highlights: make_highlights(2),
        ..Default::default()
    };
    let mut app = test_app_with_stats(RunOutcome::Won, stats);
    app.insert_resource(HighlightConfig {
        highlight_cap: 10,
        ..Default::default()
    });
    app.update();

    let texts = collect_texts(&mut app);
    let highlight_count = texts.iter().filter(|t| is_highlight_text(t)).count();
    assert_eq!(
        highlight_count, 2,
        "expected 2 highlights when only 2 exist (cap=10), got {highlight_count} in texts: {texts:?}"
    );
}
