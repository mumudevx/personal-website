use anyhow::{Context, Result};
use pulldown_cmark::{html, Parser};
use std::{fs, path::Path};
use tera::Tera;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let src_dir = "src/content";
    let dist_dir = "dist";

    // Create the dist directory
    fs::create_dir_all(dist_dir).context("Failed to create dist directory")?;

    // Initialize template engine
    let tera = Tera::new("src/template/**/*.html").context("Failed to load templates")?;

    // Process each markdown file in the src/content directory
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            process_markdown(path, dist_dir, &tera)?;
        }
    }

    copy_assets("src/assets", "dist/assets")?;

    println!("Static site generated successfully in `{}`", dist_dir);
    Ok(())
}

fn process_markdown(src_path: &Path, dist_dir: &str, tera: &Tera) -> Result<()> {
    // Read markdown file
    let content = fs::read_to_string(src_path).context("Failed to read markdown file")?;

    // Parse markdown to HTML
    let parser = Parser::new(&content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Determine output path
    let relative_path = src_path.strip_prefix("src/content")?;
    let output_path = Path::new(dist_dir)
        .join(relative_path)
        .with_extension("html");

    // Create parent directories if necessary
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).context("Failed to create output directories")?;
    }

    // Render HTML using Tera template
    let mut context = tera::Context::new();
    context.insert("content", &html_output);

    let rendered = tera
        .render("base.html", &context)
        .context("Failed to render template")?;

    // Write to the output HTML file
    fs::write(output_path, rendered).context("Failed to write HTML file")?;
    Ok(())
}

fn copy_assets(src: &str, dest: &str) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(src)?;
            let dest_path = Path::new(dest).join(relative_path);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).context("Failed to create asset directories")?;
            }
            fs::copy(path, dest_path).context("Failed to copy asset")?;
        }
    }
    Ok(())
}
