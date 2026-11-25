//! The launcher: finds the installed AutoHotkey interpreters and decides which
//! one runs a script (#Requires, syntax detection, fallback, user preferences).
//! Rules from launcher.ahk / launcher-common.ahk / identify.ahk (see MECHANICS.md).

use crate::config::Config;
use crate::sys;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Interpreter {
    pub path: PathBuf,
    pub version: String,
    pub description: String,
}

// ---------------------------------------------------------------- versions

fn split_ver(v: &str) -> (Vec<String>, Option<String>) {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let v = v.split('+').next().unwrap_or("");
    let (main, pre) = match v.find('-') {
        Some(i) => (&v[..i], Some(v[i + 1..].to_string())),
        None => (v, None),
    };
    (main.split('.').map(|s| s.to_string()).collect(), pre)
}

fn cmp_part(a: &str, b: &str) -> Ordering {
    // Numeric prefix first, then the rest as text (handles "02a0").
    let na: String = a.chars().take_while(|c| c.is_ascii_digit()).collect();
    let nb: String = b.chars().take_while(|c| c.is_ascii_digit()).collect();
    let ia: u64 = na.parse().unwrap_or(0);
    let ib: u64 = nb.parse().unwrap_or(0);
    ia.cmp(&ib).then_with(|| a[na.len()..].cmp(&b[nb.len()..]))
}

/// Compare two version strings like AutoHotkey's VerCompare (semver-like).
pub fn ver_cmp(a: &str, b: &str) -> Ordering {
    let (ma, pa) = split_ver(a);
    let (mb, pb) = split_ver(b);
    let n = ma.len().max(mb.len());
    for i in 0..n {
        let x = ma.get(i).map(|s| s.as_str()).unwrap_or("0");
        let y = mb.get(i).map(|s| s.as_str()).unwrap_or("0");
        let o = cmp_part(x, y);
        if o != Ordering::Equal {
            return o;
        }
    }
    match (pa, pb) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => {
            for (p, q) in x.split('.').zip(y.split('.')) {
                let o = match (p.parse::<u64>(), q.parse::<u64>()) {
                    (Ok(i), Ok(j)) => i.cmp(&j),
                    _ => p.cmp(q),
                };
                if o != Ordering::Equal {
                    return o;
                }
            }
            x.split('.').count().cmp(&y.split('.').count())
        }
    }
}

/// VerCompare(v, "<op>version") -> bool.
pub fn ver_test(v: &str, spec: &str) -> bool {
    let spec = spec.trim();
    let ops = ["<=", ">=", "<", ">", "="];
    for op in ops {
        if let Some(rest) = spec.strip_prefix(op) {
            let o = ver_cmp(v, rest);
            return match op {
                "<=" => o != Ordering::Greater,
                ">=" => o != Ordering::Less,
                "<" => o == Ordering::Less,
                ">" => o == Ordering::Greater,
                _ => o == Ordering::Equal,
            };
        }
    }
    ver_cmp(v, spec) == Ordering::Equal
}

pub fn major(v: &str) -> Option<u32> {
    v.trim_start_matches(['v', 'V']).split(['.', '-', '+']).next()?.parse().ok()
}

pub struct RequiresVersion {
    pub op: String,
    pub version: String,
    pub major: u32,
}

/// ParseRequiresVersion: "[op]v?<major>..." but not "32-bit"/"64-bit".
pub fn parse_requires_version(s: &str) -> Option<RequiresVersion> {
    let s = s.trim();
    let l = s.to_ascii_lowercase();
    if l == "32-bit" || l == "64-bit" {
        return None;
    }
    let op: String = s.chars().take_while(|c| matches!(c, '<' | '>' | '=')).collect();
    let rest = &s[op.len()..];
    let rest = rest.strip_prefix(['v', 'V']).unwrap_or(rest);
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    // \b after the major: next char must not be a word char
    if let Some(c) = rest[digits.len()..].chars().next() {
        if c.is_alphanumeric() || c == '_' {
            return None;
        }
    }
    let version: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
    Some(RequiresVersion { op, version, major: digits.parse().ok()? })
}

// ---------------------------------------------------------------- interpreters

fn utf16(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
}

/// Read a string from a PE file's version resource (FileVersion, FileDescription...).
pub fn pe_version_string(bytes: &[u8], key: &str) -> Option<String> {
    let mut needle = utf16(key);
    needle.extend_from_slice(&[0, 0]);
    let pos = bytes.windows(needle.len()).position(|w| w == needle.as_slice())?;
    let mut i = pos + needle.len();
    while i % 4 != 0 {
        i += 1;
    }
    while i + 1 < bytes.len() && bytes[i] == 0 && bytes[i + 1] == 0 {
        i += 2;
        if i - pos > 64 {
            return None;
        }
    }
    let mut units = Vec::new();
    while i + 1 < bytes.len() {
        let u = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        if u == 0 {
            break;
        }
        units.push(u);
        i += 2;
        if units.len() > 200 {
            break;
        }
    }
    let s = String::from_utf16_lossy(&units);
    (!s.is_empty()).then_some(s)
}

pub fn exe_info(path: &Path) -> Option<Interpreter> {
    let bytes = std::fs::read(path).ok()?;
    let mut description = pe_version_string(&bytes, "FileDescription")?;
    let version = pe_version_string(&bytes, "FileVersion")?.replace(", ", ".");
    if path.to_string_lossy().contains("_UIA") {
        description.push_str(" UIA");
    }
    Some(Interpreter { path: path.to_path_buf(), version, description })
}

fn is_usable(desc: &str, path: &Path) -> bool {
    let re = regex::Regex::new(r"^AutoHotkey.* (32|64)-bit").unwrap();
    re.is_match(desc) && !path.to_string_lossy().to_ascii_lowercase().contains("autohotkeyux.exe")
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut q = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if q && chars.peek() == Some(&'"') => {
                chars.next();
                cur.push('"');
            }
            '"' => q = !q,
            ',' if !q => out.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    out.push(cur);
    out
}

/// Rows of <root>/UX/installed-files.csv as (Hash, Version, Path, Description) maps.
pub fn read_hashes(root: &Path) -> Vec<std::collections::HashMap<String, String>> {
    let path = root.join("UX").join("installed-files.csv");
    let Ok(text) = std::fs::read_to_string(path) else { return vec![] };
    let mut lines = text.lines();
    let Some(head) = lines.next() else { return vec![] };
    let props = parse_csv_line(head.trim_start_matches('\u{feff}'));
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| props.iter().cloned().zip(parse_csv_line(l)).collect())
        .collect()
}

fn native_path(root: &Path, p: &str) -> PathBuf {
    let is_abs = p.len() > 1 && (p.as_bytes()[1] == b':' || p.starts_with("\\\\") || p.starts_with('/'));
    if is_abs {
        PathBuf::from(p)
    } else {
        let mut out = root.to_path_buf();
        for part in p.split(['\\', '/']) {
            out.push(part);
        }
        out
    }
}

/// GetUsableAutoHotkeyExes: from installed-files.csv, else by searching the folder.
pub fn usable_interpreters(root: Option<&Path>) -> Vec<Interpreter> {
    let Some(root) = root else { return vec![] };
    let mut out: Vec<Interpreter> = read_hashes(root)
        .into_iter()
        .filter_map(|row| {
            let desc = row.get("Description")?.clone();
            let path = native_path(root, row.get("Path")?);
            if !is_usable(&desc, &path) {
                return None;
            }
            Some(Interpreter { path, version: row.get("Version")?.clone(), description: desc })
        })
        .collect();
    if out.is_empty() {
        let mut stack = vec![(root.to_path_buf(), 0)];
        while let Some((dir, depth)) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if depth < 4 {
                        stack.push((p, depth + 1));
                    }
                } else {
                    let name = p.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
                    if name.starts_with("autohotkey") && name.ends_with(".exe") {
                        if let Some(info) = exe_info(&p) {
                            if is_usable(&info.description, &p) {
                                out.push(info);
                            }
                        }
                    }
                }
            }
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
    }
    out
}

/// Versions available per major (for the Launch Config lists).
pub fn versions_by_major(list: &[Interpreter]) -> (Vec<String>, Vec<String>, bool) {
    let mut v1 = Vec::new();
    let mut v2 = Vec::new();
    let mut uia = false;
    for f in list {
        let target = match major(&f.version) {
            Some(1) => &mut v1,
            Some(2) => &mut v2,
            _ => continue,
        };
        if !target.contains(&f.version) {
            target.push(f.version.clone());
        }
        if f.path.to_string_lossy().ends_with("_UIA.exe") {
            uia = true;
        }
    }
    (v1, v2, uia)
}

// ---------------------------------------------------------------- choosing

fn match_pref(desc: &str, pref: &str) -> bool {
    let d = desc.to_lowercase();
    match pref.strip_prefix('!') {
        Some(neg) => !d.contains(&neg.to_lowercase()),
        None => d.contains(&pref.to_lowercase()),
    }
}

/// LocateExeByVersion: filter by requirements, then score preferences bit by bit.
pub fn locate_exe(cfg: &Config, list: &[Interpreter], require: &str, prefer: &str) -> Option<Interpreter> {
    let mut best: Option<Interpreter> = None;
    let mut best_score: u64 = 0;
    'files: for f in list {
        let Some(fmajor) = major(&f.version) else { continue };
        for field in require.split(',').map(|s| s.trim()) {
            if field.is_empty() {
                continue;
            }
            if let Some(m) = parse_requires_version(field) {
                let spec = if m.op.is_empty() { format!(">={}", field.trim_start_matches(['v', 'V'])) } else { field.replace(['v', 'V'], "") };
                if !ver_test(&f.version, &spec) {
                    continue 'files;
                }
                if m.op.is_empty() && fmajor > m.major {
                    continue 'files;
                }
            } else if !match_pref(&f.description, field) {
                continue 'files;
            }
        }
        let section = format!("Launcher\\v{fmajor}");
        let cv = cfg.get_str(&section, "Version", "");
        let v_pref = if cv.is_empty() { "<0,".to_string() } else { format!("={cv},") };
        let build = cfg.get_str(&section, "Build", "64,!ANSI");
        let build = if build.is_empty() { "64,!ANSI".to_string() } else { build };
        let d_pref = format!(",{},{}", build, if cfg.get_bool(&section, "UIA", false) { "UIA" } else { "!UIA" });
        let all = format!("{v_pref}{prefer}{d_pref}");
        let mut score: u64 = 0;
        for field in all.split(',').map(|s| s.trim()) {
            if field.is_empty() {
                continue;
            }
            score <<= 1;
            let ok = if field.starts_with(['<', '>', '=']) { ver_test(&f.version, field) } else { match_pref(&f.description, field) };
            if ok {
                score |= 1;
            }
        }
        let better = match &best {
            None => true,
            Some(b) => best_score < score || (best_score == score && ver_cmp(&f.version, &b.version) != Ordering::Less),
        };
        if better {
            best_score = score;
            best = Some(f.clone());
        }
    }
    best
}

/// IdentifyBySyntax (simplified): count lines that only one major version accepts.
/// Returns (version or 0, reason).
pub fn identify_by_syntax(code: &str) -> (u32, String) {
    use regex::Regex;
    let v1_marks: &[(&str, &'static str)] = &[
        (r"(?i)^\s*#(NoEnv|CommentFlag|Delimiter|DerefChar|EscapeChar|If\w*)\b", "v1-dir"),
        (r"(?i)^\s*(MsgBox|Send|SendInput|Run|Sleep|SetTimer|WinActivate|WinWait|IfWinExist|IfWinNotExist|IfWinActive|StringReplace|StringSplit|StringLeft|StringRight|FileAppend|FileRead|FileDelete|Gui|GuiControl|Menu|Loop|SetWorkingDir|SendMode|SetTitleMatchMode|ControlSend|ControlClick|InputBox|ToolTip|TrayTip|Process|Hotkey|RegRead|RegWrite|IniRead|IniWrite|Click|MouseMove|SplashTextOn|Progress|EnvGet|SetBatchLines)\s*,", "v1-cmd"),
        (r"(?i)^\s*[A-Za-z_]\w*\s*=(?:[^=]|$)", "v1-ass"),
        (r"(?i)\bByRef\s+\w", "v1-ref"),
        (r"(?i)^\s*if\s+\w+\s+(not\s+)?(in|contains|between)\s", "v1-if"),
        (r"(?i)^\s*(MsgBox|Send|Run|ToolTip|SendInput)\s+[^(\s:=].*%\w+%", "v1-pct"),
    ];
    let v2_marks: &[(&str, &'static str)] = &[
        (r"=>", "v2-fat"),
        (r"(?i)^\s*#HotIf\b", "v2-dir"),
        (r"(?i)\b(MsgBox|Send|SendInput|Run|Sleep|SetTimer|WinActivate|WinWait|FileAppend|ToolTip|TrayTip|InputBox|Click|MouseMove|Hotkey|RegRead|RegWrite|IniRead|IniWrite|SetWorkingDir|SendMode)\(", "v2-fn"),
        (r"(?i)\bGui\(\)", "v2-gui"),
        (r"(?i)^\s*\w+\s*\([^()]*&\w+", "v2-ref"),
        (r":=\s*'[^']*'", "v2-sq"),
        (r"(?i)^\s*(local|global|static)\s+\w+\s*:=", "v2-kw"),
    ];
    fn comp(m: &[(&str, &'static str)]) -> Vec<(Regex, &'static str)> {
        m.iter().map(|(r, n)| (Regex::new(r).unwrap(), *n)).collect()
    }
    let (r1, r2) = (comp(v1_marks), comp(v2_marks));
    let (mut c1, mut c2) = (0u32, 0u32);
    let mut marks: Vec<&str> = Vec::new();
    let mut in_block = false;
    for raw in code.lines() {
        let t = raw.trim_start();
        if in_block {
            if t.starts_with("*/") || t.contains("*/") {
                in_block = false;
            }
            continue;
        }
        if t.starts_with("/*") {
            in_block = !t.contains("*/");
            continue;
        }
        if t.starts_with(';') || t.is_empty() {
            continue;
        }
        // strip a trailing " ; comment"
        let line = match raw.find(" ;").or_else(|| raw.find("\t;")) {
            Some(i) => &raw[..i],
            None => raw,
        };
        for (re, name) in &r1 {
            if re.is_match(line) {
                c1 += 1;
                if !marks.contains(name) {
                    marks.push(name);
                }
                break;
            }
        }
        for (re, name) in &r2 {
            if re.is_match(line) {
                c2 += 1;
                if !marks.contains(name) {
                    marks.push(name);
                }
                break;
            }
        }
    }
    let joined = marks.join(" ");
    if c1 == 0 && c2 == 0 {
        return (0, "no tell-tale matches".into());
    }
    if c1 > 0 && c2 > 0 {
        let r = if c1 > c2 { format!("v1 {c1}:{c2} - {joined}") } else if c2 > c1 { format!("v2 {c2}:{c1} - {joined}") } else { format!("? {c1}:{c2} - {joined}") };
        return (0, r);
    }
    (if c1 > 0 { 1 } else { 2 }, joined)
}

pub struct LaunchParams {
    pub exe: Option<Interpreter>,
    /// When the version could not be decided: the candidates to ask about.
    pub ask: Vec<Interpreter>,
    pub version: String,
    pub switches: Vec<String>,
    #[allow(dead_code)]
    pub require: String,
    pub error: Option<String>,
}

fn normalise_list(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c == ',').filter(|t| !t.is_empty()).collect::<Vec<_>>().join(",")
}

fn version_to_install(r: &str) -> String {
    if r.parse::<f64>().is_ok() {
        return r.to_string();
    }
    let mut v = String::new();
    for f in r.split(',').map(|s| s.trim()) {
        if let Some(m) = parse_requires_version(f) {
            if !m.op.is_empty() {
                return String::new();
            }
            v = m.version;
        }
    }
    v
}

/// GetLaunchParameters.
pub fn launch_parameters(cfg: &Config, list: &[Interpreter], script: &Path, run_with: Option<&str>) -> LaunchParams {
    let code = std::fs::read(script).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
    let re = regex::Regex::new(r"(?im)^[ \t]*#Requires[ \t]+AutoHotkey[ \t]+([^;\r\n]*?)[ \t]*(?:;[ \t]*prefer[ \t]+([^;\r\n\.]+))?(?:$|;)").unwrap();
    let mut require = String::new();
    let mut prefer = String::new();
    let mut error = None;
    if let Some(c) = re.captures(&code) {
        require = normalise_list(c.get(1).map(|m| m.as_str()).unwrap_or(""));
        prefer = normalise_list(c.get(2).map(|m| m.as_str()).unwrap_or(""));
    } else if cfg.get_bool("Launcher", "Identify", true) {
        let (v, _reason) = identify_by_syntax(&code);
        if v != 0 {
            require = v.to_string();
        }
    }
    let has_version = require.split(',').any(|f| {
        let f = f.trim();
        let l = f.to_ascii_lowercase();
        l != "32-bit" && l != "64-bit" && f.trim_start_matches(['<', '>', '=']).trim_start_matches(['v', 'V']).starts_with(|c: char| c.is_ascii_digit())
    });
    let mut hasv = has_version;
    if !hasv {
        let fb = cfg.get_str("Launcher", "Fallback", "");
        if !fb.is_empty() {
            require = if require.is_empty() { fb } else { format!("{require},{fb}") };
            hasv = true;
        }
    }
    if let Some(rw) = run_with {
        prefer = format!("{rw},{prefer}");
    }
    let prefer = prefer.trim_matches(',').to_string();
    let v = version_to_install(&require);
    let mut exe = None;
    let mut ask = Vec::new();
    if !hasv {
        for m in 1..=2 {
            let r = if require.is_empty() { m.to_string() } else { format!("{m},{require}") };
            if let Some(f) = locate_exe(cfg, list, &r, &prefer) {
                ask.push(f);
            }
        }
        if ask.len() == 1 {
            exe = ask.pop();
        }
    } else {
        exe = locate_exe(cfg, list, &require, &prefer);
        if exe.is_none() {
            let name = script.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            error = Some(format!(
                "Unable to locate the appropriate interpreter to run this script.\n\nScript:\t{name}\nRequires: {}",
                require.replace(',', " ")
            ));
        }
    }
    let mut switches = Vec::new();
    if let Some(e) = &exe {
        if major(&e.version) == Some(1) && cfg.get_bool("Launcher\\v1", "UTF8", false) {
            switches.push("/CP65001".to_string());
        }
    }
    LaunchParams { exe, ask, version: v, switches, require, error }
}

/// Run a script with an interpreter (through Wine on Linux) and wait for it.
pub fn launch_script(exe: &Path, script: &Path, args: &[String], switches: &[String]) -> i32 {
    let mut cmd = sys::windows_program_command(exe);
    cmd.extend(switches.iter().cloned());
    cmd.push(script.to_string_lossy().into_owned());
    cmd.extend(args.iter().cloned());
    match sys::spawn(&cmd, None) {
        Ok(mut child) => child.wait().ok().and_then(|s| s.code()).unwrap_or(0),
        Err(e) => {
            eprintln!("[Launcher] could not start {:?}: {e}", cmd);
            2
        }
    }
}
