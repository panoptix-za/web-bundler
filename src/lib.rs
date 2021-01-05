use anyhow::{anyhow, Context, Result};
use rand::{thread_rng, Rng};
use std::{fs, path::PathBuf, process::Command, thread, time::Duration};
use tera::Tera;
use walkdir::WalkDir;

/// Bundles a Seed SPA web application for publishing
///
/// - This script will run wasm-pack for the indicated crate.
/// - An index.html file will be read from the src_dir, and processed with the Tera templating engine.
/// - The .wasm file is versioned.
/// - Files in ./static are copied to the output without modification.
/// - Files with a .scss extension in ./css are compiled to css.
///
/// # Example index.html
/// ```html
/// <!DOCTYPE html>
/// <html lang="en">
///     <head>
///         <base href="{{ base_url }}">
///         <meta charset="utf-8">
///         <meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">
///
///         {{ stylesheet | safe }}
///
///         <title>My Amazing Website</title>
///     </head>
///     <body>
///         <div id="app"></div>
///         {{ javascript | safe }}
///     </body>
/// </html>
/// ```
pub struct WebBundlerOpt {
    /// Where to look for input files. Usually the root of the SPA crate.
    pub src_dir: PathBuf,
    /// The directory where output should be written to. In build.rs scripts, this should be read from the "OUT_DIR" environment variable.
    pub dist_dir: PathBuf,
    /// A directory that web-bundler can use to store temporary artifacts.
    pub tmp_dir: PathBuf,
    /// Passed into the index.html template as base_url. Example template usage: `<base href="{{ base_url }}">`
    pub base_url: Option<String>,
    /// Rename the webassembly bundle to include this version number.
    pub wasm_version: String,
    /// Build in release mode, instad of debug mode.
    pub release: bool,
    /// Path to the root of the workspace. A new target directory, called 'web-target' is placed there. If you aren't using a workspace, this can be wherever your `target` directory lives.
    pub workspace_root: PathBuf,
    /// Any additional directories that, if changes happen here, a rebuild is required.
    pub additional_watch_dirs: Vec<PathBuf>,
}

pub fn run(opt: WebBundlerOpt) -> Result<()> {
    list_cargo_rerun_if_changed_files(&opt)?;

    run_wasm_pack(&opt, 3)?;
    prepare_dist_directory(&opt)?;
    bundle_assets(&opt)?;
    bundle_js_snippets(&opt)?;
    bundle_index_html(&opt)?;
    bundle_app_wasm(&opt)?;
    Ok(())
}

fn list_cargo_rerun_if_changed_files(opt: &WebBundlerOpt) -> Result<()> {
    for entry in WalkDir::new(&opt.src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }
    for additional_watch_dir in &opt.additional_watch_dirs {
        for entry in WalkDir::new(&additional_watch_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
    Ok(())
}

fn run_wasm_pack(opt: &WebBundlerOpt, retries: u32) -> Result<()> {
    let target_dir = opt.workspace_root.join("web-target");
    let output = Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg(if opt.release { "--release" } else { "--dev" })
        .arg("--no-typescript")
        .arg("--out-name")
        .arg("package")
        .arg("--out-dir")
        .arg(opt.tmp_dir.as_os_str())
        .current_dir(&opt.src_dir)
        .env("CARGO_TARGET_DIR", target_dir.as_os_str())
        .output()
        .context("Failed to run wasm-pack")?;

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let is_wasm_cache_error = stderr.contains("Error: Directory not empty")
            || stderr.contains("binary does not exist");

        if is_wasm_cache_error && retries > 0 {
            // This step could error because of a legitimate failure,
            // or it could error because two parallel wasm-pack
            // processes are conflicting over WASM_PACK_CACHE. This
            // random wait in an attempt to get them restarting at
            // different times.
            let wait_ms = thread_rng().gen_range(1000..5000);
            thread::sleep(Duration::from_millis(wait_ms));
            run_wasm_pack(opt, retries - 1)
        } else {
            Err(anyhow!(
                "\
wasm-pack failed to build the package.
stdout:
{}
stderr:
{}",
                stdout,
                stderr
            ))
        }
    }
}

fn prepare_dist_directory(opt: &WebBundlerOpt) -> Result<()> {
    if opt.dist_dir.is_dir() {
        fs::remove_dir_all(&opt.dist_dir).with_context(|| {
            format!(
                "Failed to clear old dist directory ({})",
                opt.dist_dir.display()
            )
        })?;
    }
    fs::create_dir_all(&opt.dist_dir).with_context(|| {
        format!(
            "Failed to create the dist directory ({})",
            opt.dist_dir.display()
        )
    })?;
    Ok(())
}

fn bundle_assets(opt: &WebBundlerOpt) -> Result<()> {
    let src = opt.src_dir.join("static");
    let dest = &opt.dist_dir;
    if src.exists() {
        fs_extra::dir::copy(&src, &dest, &fs_extra::dir::CopyOptions::new()).with_context(
            || {
                format!(
                    "Failed to copy static files from {} to {}",
                    src.display(),
                    dest.display()
                )
            },
        )?;
    }
    Ok(())
}

fn bundle_index_html(opt: &WebBundlerOpt) -> Result<()> {
    let src_index_path = opt.src_dir.join("index.html");
    let index_html_template = fs::read_to_string(&src_index_path).with_context(|| {
        format!(
            "Failed to read {}. This should be a source code file checked into the repo.",
            src_index_path.display()
        )
    })?;

    let mut tera_context = tera::Context::new();

    let package_js_path = opt.tmp_dir.join("package.js");
    let package_js_content = fs::read_to_string(&package_js_path).with_context(|| {
        format!(
            "Failed to read {}. This should have been produced by wasm-pack",
            package_js_path.display()
        )
    })?;
    let javascript = format!(
        r#"<script type="module">{} init('app-{}.wasm'); </script>"#,
        package_js_content, opt.wasm_version
    );
    tera_context.insert("javascript", &javascript);

    tera_context.insert("base_url", opt.base_url.as_deref().unwrap_or("/"));

    let sass_options = sass_rs::Options {
        output_style: sass_rs::OutputStyle::Compressed,
        precision: 4,
        indented_syntax: true,
        include_paths: Vec::new(),
    };
    let style_src_path = opt.src_dir.join("css/style.scss");
    let style_css_content = sass_rs::compile_file(&style_src_path, sass_options)
        .map_err(|e| anyhow!("Sass compilation failed: {}", e))?;

    let stylesheet = format!("<style>{}</style>", style_css_content);
    tera_context.insert("stylesheet", &stylesheet);

    let index_html_content = Tera::one_off(&index_html_template, &tera_context, true)?;

    let dest_index_path = opt.dist_dir.join("index.html");
    fs::write(&dest_index_path, index_html_content).with_context(|| {
        format!(
            "Failed to write the index.html file to {}",
            dest_index_path.display()
        )
    })?;

    Ok(())
}

fn bundle_app_wasm(opt: &WebBundlerOpt) -> Result<()> {
    let src = opt.tmp_dir.join("package_bg.wasm");
    let dest = opt.dist_dir.join(format!("app-{}.wasm", opt.wasm_version));
    fs::copy(&src, &dest).with_context(|| {
        format!(
            "Failed to copy application wasm from {} to {}",
            src.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn bundle_js_snippets(opt: &WebBundlerOpt) -> Result<()> {
    let src = opt.tmp_dir.join("snippets");
    let dest = &opt.dist_dir;

    if src.exists() {
        fs_extra::dir::copy(&src, &dest, &fs_extra::dir::CopyOptions::new()).with_context(
            || {
                format!(
                    "Failed to copy js snippets from {} to {}",
                    src.display(),
                    dest.display()
                )
            },
        )?;
    }
    Ok(())
}
