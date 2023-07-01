//! Generate HTML pages for browsing the Ask Historians Reddit Archive.

// clippy WARN level lints
#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::map_err_ignore,
    clippy::panic,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable
)]
// clippy WARN level lints, that can be upgraded to DENY if preferred
#![warn(
    clippy::float_arithmetic,
    clippy::integer_arithmetic,
    clippy::modulo_arithmetic,
    clippy::as_conversions,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::string_slice,
    clippy::try_err
)]
// clippy DENY level lints, they always have a quick fix that should be preferred
#![deny(
    clippy::wildcard_imports,
    clippy::multiple_inherent_impl,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::separated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_to_string,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads
)]

mod config;

use std::collections::HashMap;
use std::io::BufRead;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use askama::Template;
pub use config::CONFIG;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::{debug, info};

#[tracing::instrument]
pub fn run() -> Result<()> {
    debug!("Reading posts");
    let mut posts = read_posts();
    info!("Posts w/ num_comments > 0: {}", posts.len());

    debug!("Reading comments");
    read_comments(&mut posts)?;

    debug!("Cleaning up comments");
    std::fs::remove_dir_all("output")?;
    std::fs::create_dir_all("output/posts")?;

    // Calculate real number of comments
    posts
        .iter_mut()
        .for_each(|(_id, p)| p.num_comments = p.comments.len());

    // Remove posts without comments
    posts.retain(|_id, post| post.num_comments > 0);

    let mut rendered_posts: usize = 0;

    debug!("Rendering posts");
    for post in posts.values() {
        render_post(post)?;
        rendered_posts = rendered_posts
            .checked_add(1)
            .expect("Failed to increment post render counter");
    }

    info!("Rendered posts: {rendered_posts}");

    debug!("Rendering index");
    render_index(posts.values())?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    title: String,
    id: PostId,
    selftext: String,
    num_comments: usize,
    #[serde(deserialize_with = "deserialize_null_default")]
    selftext_html: String,
    #[serde(default)]
    comments: Vec<Comment>,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

type PostId = String;
type Posts = HashMap<PostId, Post>;

#[tracing::instrument]
fn read_posts() -> Posts {
    let lines = std::io::BufReader::new(
        std::fs::File::open("submissions.json").expect("Could not read submissions.json"),
    )
    .lines()
    .take(5_000)
    .collect::<Vec<_>>();

    lines
        .into_par_iter()
        .map(|maybe_line| {
            let line = maybe_line.expect("could not read line");

            let post: Post = serde_json::from_str(&line)
                .context(line)
                .expect("could not deserialize");

            (post.id.clone(), post)
        })
        .filter(|(_id, post)| post.num_comments > 0)
        .map(|(id, post)| (id, unescape_html(post)))
        .collect()
}

fn unescape_html(post: Post) -> Post {
    let selftext_html = html_escape::decode_html_entities(&post.selftext_html).to_string();

    Post {
        selftext_html,
        ..post
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Comment {
    parent_id: String,
    body: String,
}

#[tracing::instrument]
fn read_comments(posts: &mut Posts) -> Result<()> {
    let lines = std::io::BufReader::new(
        std::fs::File::open("comments.json").context("Could not read comments.json")?,
    )
    .lines()
    .take(50_000)
    .collect::<Vec<_>>();

    let posts_wrapper = Arc::new(Mutex::new(posts));

    lines
        .into_par_iter()
        .map(|maybe_line: Result<String, std::io::Error>| -> Comment {
            let line = maybe_line.expect("could not read line");

            let mut comment: Comment =
                serde_json::from_str(&line).expect("could not deserialize comment");
            comment.parent_id = comment.parent_id.replace("t3_", "");
            comment
        })
        .filter(|comment| comment.body != "[deleted]")
        .for_each(|comment| {
            if let Some(post) = posts_wrapper
                .lock()
                .expect("Failed to get lock")
                .get_mut(&comment.parent_id)
            {
                post.comments.push(comment);
            }
        });

    Ok(())
}

#[derive(Template)]
#[template(path = "index.jinja")]
struct IndexTemplate<'a> {
    posts: Vec<&'a Post>,
}

fn render_index<'a, P>(posts: P) -> Result<()>
where
    P: Iterator<Item = &'a Post>,
{
    let template = IndexTemplate {
        posts: posts.collect(),
    };

    let output = template.render()?;

    std::fs::write("output/index.html", output)?;

    Ok(())
}

#[derive(Template)]
#[template(path = "post.jinja")]
struct PostTemplate<'a> {
    post: &'a Post,
}

fn render_post(post: &Post) -> Result<()> {
    let template = PostTemplate { post };

    let output = template.render()?;
    let name = &post.id;
    std::fs::write(format!("output/posts/{name}.html"), output)?;

    Ok(())
}
