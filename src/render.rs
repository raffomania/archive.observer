use anyhow::Result;
use askama::Template;
use chrono::{DateTime, Utc};

pub struct Comment {
    pub body: String,
}

pub struct Post {
    pub selftext_html: Option<String>,
    pub selftext: String,
    pub id: String,
    pub title: String,
    pub real_num_comments: usize,
    pub comments: Vec<Comment>,
    pub created_at: DateTime<Utc>,
}

#[derive(Template)]
#[template(path = "listing.jinja")]
struct ListingTemplate<'a> {
    posts: &'a [Post],
    page: usize,
    last_page: usize,
    next_page: Option<usize>,
    previous_page: Option<usize>,
}

pub fn listing(posts: &[Post], page: usize, total_pages: usize) -> Result<()> {
    let last_page = total_pages - 1;
    let next_page = (page < last_page).then(|| page + 1);
    let previous_page = (page > 1).then(|| page - 1);
    let template = ListingTemplate {
        posts,
        page,
        last_page,
        next_page,
        previous_page,
    };

    let output = template.render()?;

    std::fs::write(format!("output/pages/{page}.html"), output)?;

    Ok(())
}

#[derive(Template)]
#[template(path = "post.jinja")]
struct PostTemplate<'a> {
    post: &'a Post,
}

pub fn post(post: &Post) -> Result<()> {
    let template = PostTemplate { post };

    let output = template.render()?;
    let name = &post.id;
    std::fs::write(format!("output/posts/{name}.html"), output)?;

    Ok(())
}

#[derive(Template)]
#[template(path = "index.jinja")]
struct IndexTemplate;

pub fn index() -> Result<()> {
    let template = IndexTemplate {};

    let output = template.render()?;
    std::fs::write("output/index.html", output)?;

    Ok(())
}

#[derive(Template)]
#[template(path = "about.jinja")]
struct AboutTemplate {
    post_limit: String,
}

pub fn about(limit: DateTime<Utc>) -> Result<()> {
    let post_limit = limit.format("%F").to_string();
    let template = AboutTemplate { post_limit };

    let output = template.render()?;
    std::fs::write("output/about.html", output)?;

    Ok(())
}
