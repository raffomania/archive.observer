use anyhow::Result;
use askama::Template;

pub struct Comment {
    pub body: String,
}

pub struct Post {
    pub selftext_html: String,
    pub id: String,
    pub title: String,
    pub real_num_comments: usize,
    pub comments: Vec<Comment>,
}

#[derive(Template)]
#[template(path = "index.jinja")]
struct IndexTemplate {
    posts: Vec<Post>,
}

pub fn index(posts: Vec<Post>) -> Result<()> {
    let template = IndexTemplate { posts };

    let output = template.render()?;

    std::fs::write("output/index.html", output)?;

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
