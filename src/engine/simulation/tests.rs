use crate::data::GameContent;

use super::SimulationRunner;

#[test]
fn starter_series_produces_finished_reports() {
    let content = GameContent::load().expect("content loads");
    let report = SimulationRunner::run_starter_series(&content, 30);

    assert_eq!(
        report.matches_run,
        content.starter_loadouts.len() * content.starter_loadouts.len() * 2
    );
    assert!(report.player_a_wins + report.player_b_wins > 0);
    assert!(report
        .reports
        .iter()
        .all(|entry| entry.final_climax_declared));
    assert!(report
        .reports
        .iter()
        .all(|entry| !entry.encounters.is_empty()));
    assert!(report.reports.iter().all(|entry| !entry.actions.is_empty()));
    assert!(report
        .reports
        .iter()
        .all(|entry| entry.non_final_climax_rounds < entry.rounds_completed));
}
