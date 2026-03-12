use chrono::{DateTime, Datelike, Local};
use glob::glob;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const CONTENT_DIR: &str = "../content";
const INDEX_FILE: &str = "../content/_index.md";
const ARCHIVES_DIR: &str = "../content/archives";
const ARCHIVES_FILE: &str = "../content/archives/_index.md";
const FRONTMATTER_DELIMITER: &str = "---";

#[derive(Debug)]
struct Post {
    title: String,
    url: String,
    date: DateTime<Local>,
}

#[derive(Deserialize)]
struct Frontmatter {
    title: Option<String>,
    date: Option<String>,
}

fn escape_markdown(text: &str) -> String {
    text.replace("[", "\\[").replace("]", "\\]")
}

fn extract_frontmatter(content: &str) -> Option<Frontmatter> {
    let mut parts = content.splitn(3, FRONTMATTER_DELIMITER);

    parts.next()?; // before ---
    let yaml = parts.next()?; // yaml
    parts.next()?; // after ---

    serde_yaml::from_str(yaml).ok()
}

fn parse_post(path: &Path) -> Option<Post> {
    let content = fs::read_to_string(path).ok()?;
    let frontmatter = extract_frontmatter(&content)?;

    let title = frontmatter.title?;
    let date_str = frontmatter.date?;

    let parsed = DateTime::parse_from_rfc3339(&date_str)
        .ok()?
        .with_timezone(&Local);

    let relative = path.strip_prefix(CONTENT_DIR).ok()?;

    let url = relative.parent()?.to_string_lossy().to_string() + "/";

    Some(Post {
        title,
        url,
        date: parsed,
    })
}

fn collect_posts(include_future: bool) -> Vec<Post> {
    let now = Local::now();
    let pattern = format!("{}/**/index.md", CONTENT_DIR);

    let mut posts: Vec<Post> = glob(&pattern)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|p| !p.ends_with("_index.md") && !p.to_string_lossy().contains("/archives/"))
        .filter_map(|p| parse_post(&p))
        .filter(|post| include_future || post.date <= now)
        .collect();

    posts.sort_by(|a, b| b.date.cmp(&a.date));

    posts
}

fn group_by_month(posts: Vec<Post>) -> BTreeMap<(i32, u32), Vec<Post>> {
    let mut map = BTreeMap::new();

    for post in posts {
        let key = (post.date.year(), post.date.month());
        map.entry(key).or_insert_with(Vec::new).push(post);
    }

    map
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

fn render_months(grouped: &BTreeMap<(i32, u32), Vec<Post>>, url_prefix: &str) -> Vec<String> {
    let mut lines = Vec::new();

    for ((year, month), posts) in grouped.iter().rev() {
        lines.push(format!("## {} - {}\n", year, month_name(*month)));

        for post in posts {
            lines.push(format!(
                "- [{}]({}{})",
                escape_markdown(&post.title),
                url_prefix,
                post.url
            ));
        }

        lines.push(String::new());
    }

    lines
}

fn generate_index(grouped: &BTreeMap<(i32, u32), Vec<Post>>) -> String {
    let mut lines = vec![format!(
        "{}\ntitle: Blog\n{}\n",
        FRONTMATTER_DELIMITER, FRONTMATTER_DELIMITER
    )];

    lines.extend(render_months(grouped, ""));
    lines.push("[Arquivo completo →](/archives/)\n".into());

    lines.join("\n")
}

fn generate_archives(grouped: &BTreeMap<(i32, u32), Vec<Post>>) -> String {
    let mut lines = vec![format!(
        "{}\ntitle: Blog - Arquivo\n{}\n",
        FRONTMATTER_DELIMITER, FRONTMATTER_DELIMITER
    )];

    lines.extend(render_months(grouped, "/"));
    lines.join("\n")
}

fn main() {
    let include_future = std::env::args().any(|a| a == "--future");
    let cutoff_year = Local::now().year() - 1;

    let posts = collect_posts(include_future);
    let grouped = group_by_month(posts);

    let mut recent = BTreeMap::new();
    let mut archived = BTreeMap::new();

    for ((year, month), posts) in grouped {
        if year >= cutoff_year {
            recent.insert((year, month), posts);
        } else {
            archived.insert((year, month), posts);
        }
    }

    if !Path::new(ARCHIVES_DIR).exists() {
        fs::create_dir_all(ARCHIVES_DIR).unwrap();
    }

    let index_content = generate_index(&recent);
    fs::write(INDEX_FILE, index_content).unwrap();

    let recent_count: usize = recent.values().map(|v| v.len()).sum();

    println!(
        "Generated {} with {} posts ({}+).{}",
        INDEX_FILE,
        recent_count,
        cutoff_year,
        if include_future {
            " (including future posts)"
        } else {
            ""
        }
    );

    let archives_content = generate_archives(&archived);
    fs::write(ARCHIVES_FILE, archives_content).unwrap();

    let archived_count: usize = archived.values().map(|v| v.len()).sum();

    println!(
        "Generated {} with {} posts (before {}).{}",
        ARCHIVES_FILE,
        archived_count,
        cutoff_year,
        if include_future {
            " (including future posts)"
        } else {
            ""
        }
    );

    println!(
        "Me desculpa por debochar do seu projetinho de blog em Rust que você criou. Não para ganhar dinheiro, mas para estudar e usar a linguagem que gosta. Afinal, é seu direito. Fui insensível com você e com tudo que você viveu de falta de comida, moradia, educação e violência. \nContinue estudando Rust. Acesse os conhecimentos mais profundo da computação que o tempo todo o mercado nega ao sofisticar a automação do trabalho em prol da precarização e redução de custos. \n\n Novamente, minhas sinceras desculpas por ter sido escroto e debochado do Bloguinho de Rust 😔"
    );
}
