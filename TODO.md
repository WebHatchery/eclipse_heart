# TODO — Eclipse Heart

## Standards and reliability

- [ ] Migrate all 15 `src/**/tests.rs` suites and test-only helpers into crate-root `tests/`; remove test declarations under `src/` and use intentional public library APIs. Consolidate related cases toward five per major feature, preserving distinct regressions and explaining necessary exceptions (§11).
- [ ] Split large modules by responsibility before extending them: `game.rs` (783 lines), `screens/battle.rs` (774), `engine/match_engine.rs` (761), `screens/deck_builder/preview.rs` (724), `state/match_state_setup.rs` (674), and `state/decks.rs` (604). Keep the source gate's exception list empty; migrate affected legacy `mod.rs` roots to named module files during restructuring (§2.2–2.3).
- [ ] Extract functions exceeding 100 lines, starting with `Game::handle_deck_builder_action`, `DeckBuilderScreen::update`, and `update_import_export_dialog`; replace excessive argument lists with cohesive context structs and remove or narrowly justify the undocumented crate-wide Clippy allowance in `src/lib.rs` (§4, §10.2).
- [ ] Add semantic validation to `GameContent` loading for duplicate IDs, character/card/loadout references, campaign links/rewards, roster sizes, and rule invariants; return contextual errors before match setup reaches indexing or `expect` calls (§5.3, §6).
- [ ] Remove duplicated native/WASM content-loading branches in `src/data/loader.rs` by using the toolkit's platform-aware loading APIs; retain project schemas and semantic validation (§5.3).
- [ ] Handle save failures in `src/game.rs` and load failures in `src/state/app_state.rs`: report errors, offer visible retry/recovery, and prevent a failed load from silently becoming replacement default saves (§6, §7.5).
- [ ] Move remaining player-facing labels and balance constants into asset JSON, including phase/stage labels, deck-browser group labels, and the ten-card booster size; centralize label lookup through the text catalog (§5.3).
- [ ] Add missing module-purpose docs to entry points and extracted modules such as `src/game/*`, `src/screens/deck_builder/*`, and `src/state/match_state_*.rs`; correct the obsolete “non-test lines” description in `tests/code_standards.rs` (§9.2, §2.2).

## Browser controls and UI

- [ ] Add a visible pause/menu button that dispatches `ToggleEscapeMenu`; route Escape to an active dialog before toggling the global menu. Update `game_page.json`, README controls, and prompts to name equivalent visible touch controls (§7.5).
- [ ] Provide browser text entry and clipboard support for deck rename/search/metadata/import/export through shared toolkit capabilities; replace the WASM clipboard functions that always return `Err(())` in `screens/deck_builder/utils.rs` and verify operation without a physical keyboard (§7.5).
- [ ] Replace ordinary mouse-down actions in `ui/card_widgets.rs` and screen hit handlers with toolkit release-based input, documenting any intentional press actions. Replace the hidden thread-local button queue with explicitly owned UI state and separate drawing from input handling (§5.1, §7).
- [ ] Reproduce and fix reported deck-select/campaign-start overlaps, including growing deck/run lists; verify at 2560×1440, common smaller desktop sizes, and touch layouts. Store captures directly in `docs/verification/`, replacing captures of the same state (§7.5, §12).
- [ ] Add tap-accessible character inspection before selecting Magical Girls, Baddies, or support pairs in setup and campaign selection screens.

## Campaign and content

- [ ] Add a post-battle results screen with victory/defeat, Continue, and a choice from all encounter `reward_story_card_ids`; replace the automatic first-card grant in `Game::resolve_campaign_victory` and apply the chosen reward exactly once.
- [ ] Expand `magical_girl_campaign.json` beyond three linear nodes into narrative acts with a boss per act and story transitions between encounters.
- [ ] Add persistent starter unlocks, difficulty variants, and selectable alternate campaign routes.
- [ ] Improve `src/engine/ai.rs` beyond first-playable-card selection with card sequencing, reveal timing, and Final Climax race evaluation.
- [ ] Extend character JSON schemas and engine resolution with reveal, transformed, and final-form abilities.
- [ ] Specify and implement the unresolved interactions in `docs/RULES.md` §16: simultaneous upgrades/downgrades, Final Climax replacement effects, and card-specific hidden-support/exhaustion exceptions.

## Regression coverage

- [ ] After migrating legacy suites, add public-action regression coverage for exactly-once campaign rewards and roster/preview/filter interactions that must not accidentally change support-card contents; preserve existing draw, discard, deck-editing, and setup coverage.
- [ ] Add a deterministic combat replay fixture using the initial decks, enemy, hand, action sequence, and seed where randomness is involved; assert the same final state and event log on replay.
