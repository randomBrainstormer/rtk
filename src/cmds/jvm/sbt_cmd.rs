use crate::core::tracking;
use crate::core::utils::{resolved_command, truncate};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::ffi::OsString;

lazy_static! {
    /// Matches the ScalaTest summary line:
    /// Tests: succeeded N, failed N, canceled N, ignored N, pending N
    static ref TEST_SUMMARY_RE: Regex = Regex::new(
        r"Tests: succeeded (\d+), failed (\d+), canceled (\d+), ignored (\d+), pending (\d+)"
    ).unwrap();

    /// Matches suite count line:
    /// Suites: completed N, aborted N
    static ref SUITE_SUMMARY_RE: Regex = Regex::new(
        r"Suites: completed (\d+), aborted (\d+)"
    ).unwrap();

    /// Matches the "Run completed in" timing line
    static ref RUN_TIME_RE: Regex = Regex::new(
        r"Run completed in (\d+) seconds?"
    ).unwrap();

    /// Matches [info] Compiling N Scala source(s)
    static ref COMPILE_COUNT_RE: Regex = Regex::new(
        r"\[info\] Compiling (\d+) Scala source"
    ).unwrap();

    /// Matches [success] Total time: Ns
    static ref SUCCESS_TIME_RE: Regex = Regex::new(
        r"\[success\] Total time: (\d+) s"
    ).unwrap();

    /// Matches [error] lines
    static ref ERROR_RE: Regex = Regex::new(
        r"^\[error\]"
    ).unwrap();

    /// Lines that are SBT noise (loading, resolving, downloading, etc.)
    static ref NOISE_RE: Regex = Regex::new(
        r"^\[info\] (welcome to sbt|loading |set current project|Updating |Resolved |Fetching |downloading |Done )"
    ).unwrap();
}

pub fn run_test(args: &[String], verbose: u8) -> Result<()> {
    let timer = tracking::TimedExecution::start();

    let mut cmd = resolved_command("sbt");
    cmd.arg("test");

    for arg in args {
        cmd.arg(arg);
    }

    if verbose > 0 {
        eprintln!("Running: sbt test {}", args.join(" "));
    }

    let output = cmd
        .output()
        .context("Failed to run sbt test. Is SBT installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = format!("{}\n{}", stdout, stderr);

    let exit_code = output
        .status
        .code()
        .unwrap_or(if output.status.success() { 0 } else { 1 });
    let filtered = filter_sbt_test(&raw);

    if let Some(hint) = crate::core::tee::tee_and_hint(&raw, "sbt_test", exit_code) {
        println!("{}\n{}", filtered, hint);
    } else {
        println!("{}", filtered);
    }

    timer.track(
        &format!("sbt test {}", args.join(" ")),
        &format!("rtk sbt test {}", args.join(" ")),
        &raw,
        &filtered,
    );

    if !output.status.success() {
        std::process::exit(exit_code);
    }

    Ok(())
}

pub fn run_compile(args: &[String], verbose: u8) -> Result<()> {
    let timer = tracking::TimedExecution::start();

    let mut cmd = resolved_command("sbt");
    cmd.arg("compile");

    for arg in args {
        cmd.arg(arg);
    }

    if verbose > 0 {
        eprintln!("Running: sbt compile {}", args.join(" "));
    }

    let output = cmd
        .output()
        .context("Failed to run sbt compile. Is SBT installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = format!("{}\n{}", stdout, stderr);

    let exit_code = output
        .status
        .code()
        .unwrap_or(if output.status.success() { 0 } else { 1 });
    let filtered = filter_sbt_compile(&raw);

    if let Some(hint) = crate::core::tee::tee_and_hint(&raw, "sbt_compile", exit_code) {
        if !filtered.is_empty() {
            println!("{}\n{}", filtered, hint);
        } else {
            println!("{}", hint);
        }
    } else if !filtered.is_empty() {
        println!("{}", filtered);
    }

    timer.track(
        &format!("sbt compile {}", args.join(" ")),
        &format!("rtk sbt compile {}", args.join(" ")),
        &raw,
        &filtered,
    );

    if !output.status.success() {
        std::process::exit(exit_code);
    }

    Ok(())
}

pub fn run_run(args: &[String], verbose: u8) -> Result<()> {
    let timer = tracking::TimedExecution::start();

    let mut cmd = resolved_command("sbt");
    cmd.arg("run");

    for arg in args {
        cmd.arg(arg);
    }

    if verbose > 0 {
        eprintln!("Running: sbt run {}", args.join(" "));
    }

    let output = cmd
        .output()
        .context("Failed to run sbt run. Is SBT installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = format!("{}\n{}", stdout, stderr);

    let exit_code = output
        .status
        .code()
        .unwrap_or(if output.status.success() { 0 } else { 1 });
    let filtered = filter_sbt_run(&raw);

    if let Some(hint) = crate::core::tee::tee_and_hint(&raw, "sbt_run", exit_code) {
        println!("{}\n{}", filtered, hint);
    } else {
        println!("{}", filtered);
    }

    timer.track(
        &format!("sbt run {}", args.join(" ")),
        &format!("rtk sbt run {}", args.join(" ")),
        &raw,
        &filtered,
    );

    if !output.status.success() {
        std::process::exit(exit_code);
    }

    Ok(())
}

pub fn run_other(args: &[OsString], verbose: u8) -> Result<()> {
    if args.is_empty() {
        anyhow::bail!("sbt: no subcommand specified");
    }

    let timer = tracking::TimedExecution::start();

    let subcommand = args[0].to_string_lossy();
    let mut cmd = resolved_command("sbt");
    cmd.arg(&*subcommand);

    for arg in &args[1..] {
        cmd.arg(arg);
    }

    if verbose > 0 {
        eprintln!("Running: sbt {} ...", subcommand);
    }

    let output = cmd
        .output()
        .with_context(|| format!("Failed to run sbt {}", subcommand))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = format!("{}\n{}", stdout, stderr);

    print!("{}", stdout);
    eprint!("{}", stderr);

    timer.track(
        &format!("sbt {}", subcommand),
        &format!("rtk sbt {}", subcommand),
        &raw,
        &raw,
    );

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }

    Ok(())
}

/// Filter SBT test output (ScalaTest format).
///
/// On success: compact single-line summary.
/// On failure: show failed test details + summary.
fn filter_sbt_test(output: &str) -> String {
    let mut succeeded: u32 = 0;
    let mut failed: u32 = 0;
    let mut ignored: u32 = 0;
    let mut canceled: u32 = 0;
    let mut pending: u32 = 0;
    let mut suites: u32 = 0;
    let mut run_time_secs: Option<u32> = None;
    let mut has_summary = false;

    // Collect failure details
    let mut failure_lines: Vec<String> = Vec::new();
    let mut error_lines: Vec<String> = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Parse the ScalaTest summary line
        if let Some(caps) = TEST_SUMMARY_RE.captures(trimmed) {
            succeeded = caps[1].parse().unwrap_or(0);
            failed = caps[2].parse().unwrap_or(0);
            canceled = caps[3].parse().unwrap_or(0);
            ignored = caps[4].parse().unwrap_or(0);
            pending = caps[5].parse().unwrap_or(0);
            has_summary = true;
            continue;
        }

        // Parse suite count
        if let Some(caps) = SUITE_SUMMARY_RE.captures(trimmed) {
            suites = caps[1].parse().unwrap_or(0);
            continue;
        }

        // Parse run time
        if let Some(caps) = RUN_TIME_RE.captures(trimmed) {
            run_time_secs = caps[1].parse().ok();
            continue;
        }

        // Collect failed test lines (*** FAILED ***)
        if trimmed.contains("*** FAILED ***") {
            failure_lines.push(trimmed.to_string());
            continue;
        }

        // Collect [error] lines
        if ERROR_RE.is_match(trimmed) {
            let error_text = trimmed.strip_prefix("[error] ").unwrap_or(trimmed);
            if !error_text.is_empty() {
                error_lines.push(error_text.to_string());
            }
        }
    }

    // If no summary found, return a minimal fallback
    if !has_summary {
        if !error_lines.is_empty() {
            let mut result = String::from("sbt test: parse error\n");
            result.push_str("═══════════════════════════════════════\n");
            for line in error_lines.iter().take(20) {
                result.push_str(&format!("  {}\n", truncate(line, 120)));
            }
            return result.trim().to_string();
        }
        if output.trim().is_empty() {
            return "sbt test: No test output".to_string();
        }
        return output.to_string();
    }

    let time_str = run_time_secs.map(|s| format!("{}s", s)).unwrap_or_default();

    // All passed
    if failed == 0 && canceled == 0 {
        let mut summary = format!("sbt test: {} passed", succeeded);
        if ignored > 0 {
            summary.push_str(&format!(", {} ignored", ignored));
        }
        if pending > 0 {
            summary.push_str(&format!(", {} pending", pending));
        }
        if suites > 0 {
            summary.push_str(&format!(" ({} suites", suites));
        }
        if !time_str.is_empty() {
            if suites > 0 {
                summary.push_str(&format!(", {}", time_str));
            } else {
                summary.push_str(&format!(" ({})", time_str));
            }
        }
        if suites > 0 {
            summary.push(')');
        }
        return summary;
    }

    // Failures present
    let mut result = format!("sbt test: {} passed, {} failed", succeeded, failed);
    if ignored > 0 {
        result.push_str(&format!(", {} ignored", ignored));
    }
    if !time_str.is_empty() {
        result.push_str(&format!(" ({})", time_str));
    }
    result.push('\n');
    result.push_str("═══════════════════════════════════════\n");

    // Show failure details
    for line in &failure_lines {
        result.push_str(&format!("  [FAIL] {}\n", truncate(line, 120)));
    }

    // Show error lines (failed suites, etc.)
    if !error_lines.is_empty() {
        result.push('\n');
        for line in error_lines.iter().take(20) {
            result.push_str(&format!("  {}\n", truncate(line, 120)));
        }
    }

    result.trim().to_string()
}

/// Filter SBT compile output.
///
/// On success: compact summary with source count and time.
/// On failure: show all [error] lines.
fn filter_sbt_compile(output: &str) -> String {
    let mut source_count: u32 = 0;
    let mut total_time_secs: Option<u32> = None;
    let mut error_lines: Vec<String> = Vec::new();
    let mut has_success = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Count compiled sources
        if let Some(caps) = COMPILE_COUNT_RE.captures(trimmed) {
            source_count += caps[1].parse::<u32>().unwrap_or(0);
            continue;
        }

        // Parse success time
        if let Some(caps) = SUCCESS_TIME_RE.captures(trimmed) {
            total_time_secs = caps[1].parse().ok();
            has_success = true;
            continue;
        }

        // Collect [error] lines
        if ERROR_RE.is_match(trimmed) {
            let error_text = trimmed.strip_prefix("[error] ").unwrap_or(trimmed);
            if !error_text.is_empty() {
                error_lines.push(error_text.to_string());
            }
        }
    }

    // Compilation errors
    if !error_lines.is_empty() {
        let mut result = format!("sbt compile: {} errors\n", error_lines.len());
        result.push_str("═══════════════════════════════════════\n");

        for (i, error) in error_lines.iter().take(30).enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, truncate(error, 120)));
        }

        if error_lines.len() > 30 {
            result.push_str(&format!("\n... +{} more errors\n", error_lines.len() - 30));
        }

        return result.trim().to_string();
    }

    // Success
    if has_success || source_count > 0 {
        let mut summary = String::from("sbt compile: ");
        if source_count > 0 {
            summary.push_str(&format!("{} sources", source_count));
        } else {
            summary.push_str("Success");
        }
        if let Some(secs) = total_time_secs {
            summary.push_str(&format!(" ({}s)", secs));
        }
        return summary;
    }

    // Fallback: nothing recognized
    "sbt compile: Success".to_string()
}

/// Filter SBT run output — light filtering.
///
/// Strips SBT preamble noise, keeps actual program output.
fn filter_sbt_run(output: &str) -> String {
    let mut result_lines: Vec<&str> = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Skip empty lines at the start
        if trimmed.is_empty() && result_lines.is_empty() {
            continue;
        }

        // Skip SBT noise lines
        if NOISE_RE.is_match(trimmed) {
            continue;
        }

        // Skip [info] Compiling lines
        if COMPILE_COUNT_RE.is_match(trimmed) {
            continue;
        }

        // Skip [info] running ... preamble
        if trimmed.starts_with("[info] running ") || trimmed.starts_with("[info] Running ") {
            continue;
        }

        // Strip [info] prefix from program output
        if let Some(content) = trimmed.strip_prefix("[info] ") {
            result_lines.push(content);
        } else if let Some(content) = trimmed.strip_prefix("[success] ") {
            // Skip success time line
            if content.starts_with("Total time:") {
                continue;
            }
            result_lines.push(content);
        } else if ERROR_RE.is_match(trimmed) {
            let error_text = trimmed.strip_prefix("[error] ").unwrap_or(trimmed);
            result_lines.push(error_text);
        } else {
            result_lines.push(trimmed);
        }
    }

    result_lines.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(text: &str) -> usize {
        text.split_whitespace().count()
    }

    #[test]
    fn test_filter_sbt_test_all_pass() {
        let input = include_str!("../../../tests/fixtures/sbt/sbt_test_pass.txt");
        let output = filter_sbt_test(input);

        assert!(output.starts_with("sbt test:"));
        assert!(output.contains("30 passed"));
        assert!(output.contains("2 ignored"));
        assert!(output.contains("5 suites"));
        assert!(output.contains("5s"));
        // Should be a compact single line
        assert!(!output.contains('\n'), "All-pass output should be one line");
    }

    #[test]
    fn test_filter_sbt_test_with_failures() {
        let input = include_str!("../../../tests/fixtures/sbt/sbt_test_fail.txt");
        let output = filter_sbt_test(input);

        assert!(output.contains("15 passed"));
        assert!(output.contains("3 failed"));
        assert!(output.contains("FAIL"));
        assert!(output.contains("FAILED"));
    }

    #[test]
    fn test_filter_sbt_test_token_savings() {
        let input = include_str!("../../../tests/fixtures/sbt/sbt_test_pass.txt");
        let output = filter_sbt_test(input);

        let input_tokens = count_tokens(input);
        let output_tokens = count_tokens(&output);

        let savings = 100.0 - (output_tokens as f64 / input_tokens as f64 * 100.0);
        assert!(
            savings >= 60.0,
            "sbt test (pass) filter: expected >=60% savings, got {:.1}% (input: {}, output: {})",
            savings,
            input_tokens,
            output_tokens
        );
    }

    #[test]
    fn test_filter_sbt_test_fail_token_savings() {
        let input = include_str!("../../../tests/fixtures/sbt/sbt_test_fail.txt");
        let output = filter_sbt_test(input);

        let input_tokens = count_tokens(input);
        let output_tokens = count_tokens(&output);

        let savings = 100.0 - (output_tokens as f64 / input_tokens as f64 * 100.0);
        assert!(
            savings >= 40.0,
            "sbt test (fail) filter: expected >=40% savings, got {:.1}% (input: {}, output: {})",
            savings,
            input_tokens,
            output_tokens
        );
    }

    #[test]
    fn test_filter_sbt_compile_success() {
        let input = "[info] loading settings for project root from build.sbt ...\n\
                      [info] Compiling 15 Scala sources to /target/scala-2.13/classes ...\n\
                      [success] Total time: 12 s, completed Jan 15, 2025";
        let output = filter_sbt_compile(input);

        assert!(output.contains("sbt compile:"));
        assert!(output.contains("15 sources"));
        assert!(output.contains("12s"));
    }

    #[test]
    fn test_filter_sbt_compile_errors() {
        let input = include_str!("../../../tests/fixtures/sbt/sbt_compile_error.txt");
        let output = filter_sbt_compile(input);

        assert!(output.contains("sbt compile:"));
        assert!(output.contains("errors"));
        assert!(output.contains("type mismatch"));
        assert!(output.contains("not found: value"));
    }

    #[test]
    fn test_filter_sbt_compile_error_token_savings() {
        let input = include_str!("../../../tests/fixtures/sbt/sbt_compile_error.txt");
        let output = filter_sbt_compile(input);

        let input_tokens = count_tokens(input);
        let output_tokens = count_tokens(&output);

        let savings = 100.0 - (output_tokens as f64 / input_tokens as f64 * 100.0);
        assert!(
            savings >= 30.0,
            "sbt compile (error) filter: expected >=30% savings, got {:.1}% (input: {}, output: {})",
            savings,
            input_tokens,
            output_tokens
        );
    }

    #[test]
    fn test_filter_sbt_run_strips_noise() {
        let input = "[info] welcome to sbt 1.9.7\n\
                      [info] loading settings for project root from build.sbt ...\n\
                      [info] set current project to myapp\n\
                      [info] running com.example.Main\n\
                      [info] Hello, World!\n\
                      [info] Server started on port 8080\n\
                      [success] Total time: 3 s, completed Jan 15, 2025";
        let output = filter_sbt_run(input);

        assert!(output.contains("Hello, World!"));
        assert!(output.contains("Server started on port 8080"));
        assert!(!output.contains("welcome to sbt"));
        assert!(!output.contains("loading settings"));
        assert!(!output.contains("set current project"));
        assert!(!output.contains("running com.example"));
        assert!(!output.contains("Total time:"));
    }

    #[test]
    fn test_filter_sbt_test_empty_input() {
        let output = filter_sbt_test("");
        assert!(!output.is_empty());
    }

    #[test]
    fn test_filter_sbt_compile_empty_input() {
        let output = filter_sbt_compile("");
        assert!(output.contains("sbt compile:"));
        assert!(output.contains("Success"));
    }

    #[test]
    fn test_filter_sbt_run_empty_input() {
        let output = filter_sbt_run("");
        // Empty input produces empty output
        assert!(output.is_empty());
    }
}
