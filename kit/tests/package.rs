//! The Muse Code package (`package/`) against core: it builds reproducibly,
//! installs through the daemon's definition path, reads the captured version
//! banner, matches the captured process forms and reads every committed
//! screen (`compat/screens`) and terminal title as the case table says.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use package::directory::build_directory_archive;
use package::{read_archive, Limits};
use pohunek_daemon::agent::host::{
    definition_from_archive, InputReadiness, LaunchProgram, RuntimeDefinition,
};
use pohunek_daemon::agent::NativeReferenceStrategy;
use pohunek_daemon::detect::{ActivityTransition, DetectionConfig, Detector, DetectorConfig};
use pohunek_daemon::procwatch::{ProcessFact, StartIdentity};
use protocol::{AgentActivity, StateSource};

/// Repository root: the parent of this crate.
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate sits in the repository")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo().join(relative)).unwrap_or_else(|e| panic!("read {relative}: {e}"))
}

fn built_archive() -> Vec<u8> {
    build_directory_archive(&repo().join("package"), &Limits::DEFAULT)
        .expect("the package directory builds")
}

/// The definition `plugin install` derives from the package directory.
fn installed_definition() -> RuntimeDefinition {
    let bytes = built_archive();
    let archive = read_archive(&bytes, &Limits::DEFAULT).expect("the archive reads");
    let digest = archive.digest().clone();
    definition_from_archive(archive.entries(), &digest)
        .expect("the daemon accepts the package descriptor")
}

#[test]
fn the_package_builds_reproducibly_with_only_descriptor_and_manifest() {
    let first = built_archive();
    assert_eq!(first, built_archive(), "two builds produce the same bytes");
    let archive = read_archive(&first, &Limits::DEFAULT).expect("the archive reads");
    let paths: Vec<&str> = archive.entries().iter().map(|e| e.path.as_str()).collect();
    assert_eq!(paths, ["detect.toml", "runtime.toml"]);
}

#[test]
fn the_descriptor_declares_the_launch_contract_of_the_captured_release() {
    let definition = installed_definition();
    assert_eq!(definition.runtime_id().as_str(), "muse");
    assert_eq!(definition.display_name(), "Muse Code");
    assert_eq!(
        definition.program(),
        &LaunchProgram::Fixed("muse".to_owned())
    );
    assert_eq!(definition.default_args(), ["--trust-workspace"]);
    assert!(
        !definition.prompt_arg(),
        "the prompt is typed once the composer is drawn"
    );
    assert_eq!(
        definition.launch_env(),
        [("MUSE_NO_AUTO_UPDATE".to_owned(), "1".to_owned())]
    );
    assert_eq!(definition.input_readiness(), InputReadiness::ScreenIdle);
    assert!(definition.input_rules().bracketed_paste);
    assert_eq!(
        definition.native_reference_strategy(),
        NativeReferenceStrategy::None,
        "resume stays off until a hook capture verifies it"
    );
    assert!(definition.integration_handler().is_none());
    let program_status = definition
        .manifest()
        .program_status()
        .expect("the manifest declares program status");
    assert_eq!(program_status.app(), "muse");
    assert!(
        program_status.answer_probe(),
        "Muse sends OSC 7501 records only after its probe is answered"
    );
}

#[test]
fn the_version_probe_accepts_the_captured_banner_and_the_locked_release_only() {
    let definition = installed_definition();
    let policy = definition
        .version_probe_policy()
        .expect("the descriptor declares a version probe");
    assert_eq!(policy.args(), ["--version"]);
    let banner = read("compat/install/version.txt");
    let version = policy
        .check_output(banner.lines().next().expect("a banner line"))
        .expect("the captured banner is a supported release");
    let release = read("compat/RELEASE");
    assert!(
        release.trim().starts_with(&version.to_string()),
        "the probed release {version} is the locked release {}",
        release.trim()
    );
    for refused in [
        "Muse Code 1.4.13 (1.4.13-R2)",
        "Muse Code 1.5.0 (1.5.0-R1)",
        "Muse Code 1.4.14",
        "muse 1.4.14 (1.4.14-R7309.1)",
        "",
    ] {
        assert!(
            policy.check_output(refused).is_err(),
            "{refused:?} is not a supported banner"
        );
    }
}

fn process(comm: &str, cmdline: &[&str]) -> ProcessFact {
    ProcessFact {
        pid: 4242,
        pgid: 4242,
        ppid: 1,
        start_identity: StartIdentity::new(1),
        comm: comm.to_owned(),
        cmdline: cmdline.iter().map(|a| (*a).to_owned()).collect(),
    }
}

#[test]
fn the_process_matchers_accept_the_captured_process_and_nothing_else() {
    let definition = installed_definition();
    let matchers = definition
        .manifest()
        .process_matchers()
        .expect("the manifest declares process matchers");
    let captured = read("compat/install/process.txt");
    let (comm, rest) = captured.split_once(' ').expect("comm and arguments");
    let args: Vec<&str> = rest.split_whitespace().collect();
    assert!(
        matchers.matches(&process(comm, &args)),
        "the captured process: {comm} {args:?}"
    );
    // The kernel cuts `comm` to 15 bytes, so the name ends inside the release.
    for comm in [
        "muse",
        "muse-bin-1.4.14",
        "muse-bin-1.4.1-",
        "muse-bin-10.4.1",
    ] {
        assert!(matchers.matches(&process(comm, &["other"])), "{comm}");
    }
    let image = "/home/u/.local/bin/muse-bin-1.4.14-R7309.1";
    assert!(matchers.matches(&process("sh", &[image, "--trust-workspace"])));
    assert!(matchers.matches(&process(
        "bash",
        &["bash", "/home/u/.local/bin/muse", "resume"]
    )));
    for (comm, cmdline) in [
        ("musescore", &["musescore"][..]),
        ("amuse", &["/usr/bin/amuse"][..]),
        ("muse-bin-x", &["other"][..]),
        ("node", &["node", "/opt/muse.js"][..]),
        ("sh", &["/usr/bin/musescore", "a"][..]),
    ] {
        assert!(
            !matchers.matches(&process(comm, cmdline)),
            "{comm} {cmdline:?}"
        );
    }
}

fn frame_bytes(title: Option<&str>, rows: &[&str]) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Some(title) = title {
        bytes.extend_from_slice(format!("\x1b]2;{title}\x07").as_bytes());
    }
    bytes.extend_from_slice(b"\x1b[2J\x1b[H");
    bytes.extend_from_slice(rows.join("\r\n").as_bytes());
    bytes
}

fn detector_for(definition: &RuntimeDefinition, now: Instant, rows: u16, columns: u16) -> Detector {
    Detector::new(
        rows,
        columns,
        now,
        DetectorConfig {
            detection: DetectionConfig {
                recheck_after: Duration::from_millis(100),
                confirmations: 1,
                cap: Duration::from_millis(700),
                stable_visible_refresh: Duration::from_millis(800),
                startup_grace: Duration::ZERO,
            },
            manifest: Some(std::sync::Arc::clone(definition.manifest())),
            input_readiness: definition.input_readiness(),
        },
    )
}

fn last_with(transitions: &[ActivityTransition], source: StateSource) -> Option<AgentActivity> {
    transitions
        .iter()
        .rfind(|t| t.source == source)
        .map(|t| t.activity)
}

/// The activity the manifest reads from the screen text alone.
fn screen_activity(
    definition: &RuntimeDefinition,
    text: &str,
    columns: u16,
) -> Option<AgentActivity> {
    let rows: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
    let height = u16::try_from(rows.len().max(24)).expect("a screen height");
    let started = Instant::now();
    let mut detector = detector_for(definition, started, height, columns);
    last_with(
        &detector.feed(started, &frame_bytes(None, &rows)),
        StateSource::Screen,
    )
}

fn title_activity(definition: &RuntimeDefinition, title: &str) -> Option<AgentActivity> {
    let started = Instant::now();
    let mut detector = detector_for(definition, started, 24, 80);
    last_with(
        &detector.feed(started, format!("\x1b]2;{title}\x07").as_bytes()),
        StateSource::OscTitle,
    )
}

/// What a scenario's screens mean, at every width unless an exception names it.
struct Case {
    expect: Option<AgentActivity>,
    /// Widths whose screen legitimately reads differently, with the reason.
    exceptions: &'static [(u16, Option<AgentActivity>, &'static str)],
}

const fn all(expect: AgentActivity) -> Case {
    Case {
        expect: Some(expect),
        exceptions: &[],
    }
}

/// The case table: one row per captured scenario. A scenario with no row fails
/// the test, so a new capture needs a decision.
fn case_table() -> BTreeMap<&'static str, Case> {
    use AgentActivity::{Blocked, Idle, Working};
    BTreeMap::from([
        ("trust", all(Blocked)),
        ("login", all(Blocked)),
        ("api-key", all(Blocked)),
        ("approval", all(Blocked)),
        ("question", all(Blocked)),
        ("idle", all(Idle)),
        ("done", all(Idle)),
        ("working", all(Working)),
        ("approval-allowed", all(Working)),
        ("approval-aborted", all(Working)),
        ("clear", all(Idle)),
        ("fork", all(Idle)),
        ("seed", all(Idle)),
        ("resume", all(Idle)),
        ("update-notice", all(Working)),
    ])
}

/// Width of a screen file: `<width>.txt`, or 120 for the named resume screens.
fn width_of(path: &Path) -> u16 {
    path.file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(120)
}

#[test]
fn every_committed_screen_reads_as_the_case_table_says() {
    let definition = installed_definition();
    let table = case_table();
    let mut checked = 0;
    let mut problems = Vec::new();
    let mut scenarios: Vec<_> = fs::read_dir(repo().join("compat/screens"))
        .expect("the screens exist")
        .map(|e| e.expect("entry").path())
        .collect();
    scenarios.sort();
    for dir in scenarios {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .expect("name")
            .to_owned();
        let Some(case) = table.get(name.as_str()) else {
            problems.push(format!("scenario {name} has no case-table row"));
            continue;
        };
        let mut files: Vec<_> = fs::read_dir(&dir)
            .expect("scenario directory")
            .map(|e| e.expect("entry").path())
            .filter(|p| {
                p.extension().is_some_and(|x| x == "txt")
                    && p.file_stem().is_some_and(|s| s != "raw-head")
            })
            .collect();
        files.sort();
        for file in files {
            let columns = width_of(&file);
            let expected = case
                .exceptions
                .iter()
                .find(|(w, _, _)| *w == columns)
                .map_or(case.expect, |(_, e, _)| *e);
            let got = screen_activity(
                &definition,
                &fs::read_to_string(&file).expect("screen"),
                columns,
            );
            if got != expected {
                problems.push(format!(
                    "{name}/{}: expected {expected:?}, read {got:?}",
                    file.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                ));
            }
            checked += 1;
        }
    }
    assert!(
        problems.is_empty(),
        "{} of {checked} screens:\n{}",
        problems.len(),
        problems.join("\n")
    );
    assert!(checked >= 50, "the corpus was found ({checked} screens)");
}

#[test]
fn the_terminal_title_reads_working_only_behind_a_spinner_frame() {
    let definition = installed_definition();
    let titles = read("compat/osc7501/title-answered.txt");
    let mut seen = (0, 0);
    for title in titles.lines().filter_map(|l| l.strip_prefix("<ESC>]0;")) {
        let spinner = title
            .chars()
            .next()
            .is_some_and(|c| ('\u{2800}'..='\u{28ff}').contains(&c));
        let expected = spinner.then_some(AgentActivity::Working);
        assert_eq!(
            title_activity(&definition, title),
            expected,
            "title {title:?}"
        );
        seen = (
            seen.0 + usize::from(spinner),
            seen.1 + usize::from(!spinner),
        );
    }
    assert!(
        seen.0 > 0 && seen.1 > 0,
        "idle and working titles were captured"
    );
}

#[test]
fn prose_that_quotes_a_footer_or_a_dialog_reads_nothing() {
    let definition = installed_definition();
    let rule = "\u{2500}".repeat(40);
    for text in [
        "the profile \u{b7} read-only is described\nin this text\n".to_owned(),
        format!("{rule}\nDo you trust this workspace? Trust and continue\n\u{276f}\n{rule}\n"),
        "\u{276f} Enter to select \u{b7} \u{27e9} 1. a\n".to_owned(),
        "Log in with browser, then Set an API key\n\u{276f} explain this\n".to_owned(),
    ] {
        let read = screen_activity(&definition, &text, 80);
        assert_ne!(read, Some(AgentActivity::Blocked), "{text:?}");
    }
}

#[test]
fn a_cursor_glyph_of_a_dialog_is_not_the_idle_composer() {
    let definition = installed_definition();
    let rule = "\u{2500}".repeat(40);
    let unknown_dialog = format!("{rule}\n\u{27e9} 1. Something else\n{rule}\n");
    assert_eq!(screen_activity(&definition, &unknown_dialog, 80), None);
}
