use anyhow::{Context, Result};
use pulldown_cmark::{html, Parser};
use serde::Serialize;
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

    // Prepare listings for blog and book
    let mut blog_posts = vec![];
    let mut book_posts = vec![];

    // Process markdown files
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            let metadata = process_markdown(path, dist_dir, &tera)?;

            // Categorize the post based on its directory
            if path.starts_with("src/content/blog") {
                blog_posts.push(metadata);
            } else if path.starts_with("src/content/books") {
                book_posts.push(metadata);
            }
        }
    }

    // Generate the homepage, blog listing, and book listing pages
    generate_homepage(dist_dir, &tera)?;
    generate_listing("blog", &blog_posts, dist_dir, &tera)?;
    generate_listing("books", &book_posts, dist_dir, &tera)?;

    // Copy assets
    copy_assets("src/assets", "dist/assets")?;

    // Copy CNAME file if it exists
    if Path::new("src/CNAME").exists() {
        fs::copy("src/CNAME", "dist/CNAME").context("Failed to copy CNAME file")?;
    }

    println!("Static site generated successfully in `{}`", dist_dir);
    Ok(())
}

#[derive(Serialize)]
struct PostMetadata {
    title: String,
    slug: String,
    image: String,
    description: String,
    date: String,
}

fn process_markdown(src_path: &Path, dist_dir: &str, tera: &Tera) -> Result<PostMetadata> {
    // Read markdown file
    let content = fs::read_to_string(src_path).context("Failed to read markdown file")?;

    // Extract metadata and content, skipping frontmatter
    let (frontmatter, markdown_content) = split_frontmatter(&content);

    // Extract metadata from frontmatter
    let title = extract_metadata(&frontmatter, "title").unwrap_or_else(|| "Untitled".to_string());
    let slug = src_path.file_stem().unwrap().to_str().unwrap().to_string();
    let image = extract_metadata(&frontmatter, "image")
        .unwrap_or_else(|| "/assets/images/rubber-duck.jpg".to_string());
    let description = extract_metadata(&frontmatter, "description")
        .unwrap_or_else(|| "No description".to_string());
    let date = extract_metadata(&frontmatter, "date").unwrap_or_else(|| "No date".to_string());

    // Parse markdown to HTML (using only the content part)
    let parser = Parser::new(&markdown_content);
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

    // Determine template based on content type
    let template_name = if src_path.starts_with("src/content/blog") {
        "blog_detail.html"
    } else if src_path.starts_with("src/content/books") {
        "book_detail.html"
    } else {
        "base.html" // fallback template
    };

    // Render HTML using Tera template
    let mut context = tera::Context::new();
    context.insert("content", &html_output);
    context.insert("title", &title);
    context.insert("image", &image);
    context.insert("description", &description);
    context.insert("date", &date);
    let rendered = tera
        .render(template_name, &context)
        .context("Failed to render template")?;

    // Write to the output HTML file
    fs::write(output_path, rendered).context("Failed to write HTML file")?;
    Ok(PostMetadata {
        title,
        slug,
        image,
        description,
        date,
    })
}

fn extract_metadata(content: &str, key: &str) -> Option<String> {
    let key = format!("{}:", key);
    content
        .lines()
        .find(|line| line.starts_with(&key))
        .map(|line| line[key.len()..].trim().to_string())
}

fn generate_homepage(dist_dir: &str, tera: &Tera) -> Result<()> {
    let mut context = tera::Context::new();
    context.insert("title", "Homepage");

    let rendered = tera
        .render("homepage.html", &context)
        .context("Failed to render homepage template")?;

    let output_path = Path::new(dist_dir).join("index.html");
    fs::write(output_path, rendered).context("Failed to write homepage")?;
    Ok(())
}

fn generate_listing(
    category: &str,
    posts: &[PostMetadata],
    dist_dir: &str,
    tera: &Tera,
) -> Result<()> {
    let mut context = tera::Context::new();
    context.insert("posts", posts);
    context.insert("title", &format!("{} Listing", category));

    let template_name = match category {
        "blog" => "blog_list.html",
        "books" => "book_list.html",
        _ => return Err(anyhow::anyhow!("Unknown category: {}", category)),
    };

    let rendered = tera
        .render(template_name, &context)
        .context("Failed to render listing template")?;

    let output_path = Path::new(dist_dir).join(format!("{}.html", category));
    fs::write(output_path, rendered).context("Failed to write listing")?;
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

fn split_frontmatter(content: &str) -> (String, String) {
    let mut lines = content.lines();
    let mut frontmatter = String::new();
    let mut markdown_content = String::new();

    // Check if the file starts with frontmatter delimiter
    if let Some("---") = lines.next() {
        // Collect frontmatter until the closing delimiter
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
            frontmatter.push_str(line);
            frontmatter.push('\n');
        }

        // The rest is markdown content
        for line in lines {
            markdown_content.push_str(line);
            markdown_content.push('\n');
        }
    } else {
        // No frontmatter found, treat everything as content
        markdown_content = content.to_string();
    }

    (frontmatter, markdown_content)
}
