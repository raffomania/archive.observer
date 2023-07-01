//! Generate html pages for browsing the AskHistorians Archive.

// clippy WARN level lints
#![warn(
    clippy::cargo,
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::map_err_ignore,
    clippy::missing_docs_in_private_items,
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
    clippy::pattern_type_mismatch,
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

use std::{
    collections::HashMap,
    io::BufRead,
    sync::{Arc, Mutex},
};

pub use config::CONFIG;

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub fn run() -> Result<()> {
    let mut posts = read_posts();
    println!("Posts w/ num_comments > 0: {}", posts.len());

    read_comments(&mut posts)?;

    std::fs::remove_dir_all("output")?;
    std::fs::create_dir_all("output/posts")?;

    let parser = make_template_parser()?;

    // Calculate real number of comments
    posts
        .iter_mut()
        .for_each(|(_id, p)| p.num_comments = p.comments.len());

    // Remove posts without comments
    posts.retain(|_id, post| post.num_comments > 0);

    let mut rendered_posts = 0;

    let template = parser.parse_file("templates/post.liquid")?;
    for post in posts.values() {
        render_post(&template, post)?;
        rendered_posts += 1;
    }

    println!("Rendered posts: {rendered_posts}");

    render_index(&parser, posts.values())?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    title: String,
    id: PostId,
    selftext: String,
    num_comments: usize,
    selftext_html: Option<String>,
    #[serde(default)]
    comments: Vec<Comment>,
}

type PostId = String;
type Posts = HashMap<PostId, Post>;

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
    let selftext_html = post
        .selftext_html
        .as_ref()
        .map(html_escape::decode_html_entities)
        .map(String::from);

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
            posts_wrapper
                .lock()
                .unwrap()
                .get_mut(&comment.parent_id)
                .map(|post| post.comments.push(comment));
        });

    Ok(())
}

fn make_template_parser() -> Result<liquid::Parser> {
    let mut partial_source = liquid::partials::InMemorySource::new();

    partial_source.add("head", std::fs::read_to_string("templates/head.liquid")?);
    partial_source.add("nav", std::fs::read_to_string("templates/nav.liquid")?);
    partial_source.add(
        "layout",
        std::fs::read_to_string("templates/layout.liquid")?,
    );

    let partials = liquid::partials::EagerCompiler::new(partial_source);

    liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .context("Could not build liquid parser")
}

fn render_index<'a, P>(parser: &liquid::Parser, posts: P) -> Result<()>
where
    P: Iterator<Item = &'a Post>,
{
    let template = parser.parse_file("templates/index.liquid")?;

    let globals =
        liquid::object!({ "posts": liquid::model::to_value(&posts.collect::<Vec<_>>())? });

    let output = template.render(&globals)?;

    std::fs::write("output/index.html", output)?;

    Ok(())
}

fn render_post(template: &liquid::Template, post: &Post) -> Result<()> {
    let globals = liquid::to_object(&post)?;

    let output = template.render(&globals)?;
    let name = &post.id;
    std::fs::write(format!("output/posts/{name}.html"), output)?;

    Ok(())
}
